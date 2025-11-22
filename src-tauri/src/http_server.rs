use axum::{
    body::Body,
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Request, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use local_ip_address;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AssetResolver, Runtime};
use tokio::sync::{broadcast, mpsc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::utils::debug_log;

pub type BroadcastSender = broadcast::Sender<String>;

pub struct HttpServer<R: Runtime> {
    mdns: ServiceDaemon,
    broadcast_tx: BroadcastSender,
    asset_resolver: Option<AssetResolver<R>>,
    static_dir: Option<PathBuf>,
}

impl<R: Runtime> HttpServer<R> {
    pub fn new() -> Self {
        let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
        let (broadcast_tx, _) = broadcast::channel(100);
        HttpServer {
            mdns,
            broadcast_tx,
            asset_resolver: None,
            static_dir: None,
        }
    }

    pub fn with_asset_resolver(mut self, resolver: AssetResolver<R>) -> Self {
        self.asset_resolver = Some(resolver);
        self
    }

    pub fn broadcast(&self, message: String) {
        let _ = self.broadcast_tx.send(message);
    }

    pub async fn start(self: Arc<Self>, port: u16) {
        let addr = format!("0.0.0.0:{}", port);

        let service_type = "_http._tcp.local.";
        let instance_name = "eazycontroller";
        let host_name = "eazycontroller.local.";

        let local_ip = local_ip_address::local_ip().ok();

        match local_ip {
            Some(ip) => {
                let ip_addr = match ip {
                    std::net::IpAddr::V4(ipv4) => ipv4,
                    std::net::IpAddr::V6(_) => std::net::Ipv4Addr::new(0, 0, 0, 0),
                };

                if ip_addr != std::net::Ipv4Addr::new(0, 0, 0, 0) {
                    let service_info =
                        ServiceInfo::new(service_type, instance_name, host_name, (), port, None);

                    if let Ok(info) = service_info {
                        let info = info.enable_addr_auto();
                        let _ = self.mdns.register(info);
                    }
                }
            }
            None => {}
        }

        let app = if let Some(ref dir) = self.static_dir {
            let serve_dir = ServeDir::new(dir)
                .append_index_html_on_directories(true)
                .precompressed_gzip()
                .precompressed_br();

            Router::new()
                .route("/ws", get(ws_handler::<R>))
                .route("/health", get(health_check))
                .fallback_service(serve_dir)
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .with_state(Arc::clone(&self))
        } else {
            Router::new()
                .route("/ws", get(ws_handler::<R>))
                .route("/health", get(health_check))
                .fallback(static_file_handler::<R>)
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .with_state(Arc::clone(&self))
        };

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                debug_log!("無法綁定 HTTP 端口 {}: {:?}", port, e);
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            debug_log!("HTTP 伺服器錯誤: {:?}", e);
        }
    }
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn ws_handler<R: Runtime>(
    ws: WebSocketUpgrade,
    State(server): State<Arc<HttpServer<R>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, server))
}

async fn handle_websocket<R: Runtime>(socket: WebSocket, server: Arc<HttpServer<R>>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let mut broadcast_rx = server.broadcast_tx.subscribe();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if tx_clone.send(WsMessage::Text(msg)).is_err() {
                break;
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
                    if let Some(response) = crate::message_handler::handle_message(json).await {
                        let response_text = serde_json::to_string(&response).unwrap_or_default();
                        if tx.send(WsMessage::Text(response_text)).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    debug_log!("無法解析 JSON: {:?}", e);
                    let error_msg = serde_json::json!({
                        "type": "error",
                        "message": format!("無法解析訊息: {:?}", e)
                    });
                    let _ = tx.send(WsMessage::Text(error_msg.to_string()));
                }
            },
            Ok(WsMessage::Close(_)) => break,
            Ok(WsMessage::Ping(data)) => {
                let _ = tx.send(WsMessage::Pong(data));
            }
            Err(e) => {
                debug_log!("WebSocket 錯誤: {:?}", e);
                break;
            }
            _ => {}
        }
    }

    send_task.abort();
}

async fn static_file_handler<R: Runtime>(
    State(server): State<Arc<HttpServer<R>>>,
    request: Request,
) -> Response {
    let path = request.uri().path();

    let asset_path = path.trim_start_matches('/');

    let asset_path = if asset_path.is_empty() {
        "index.html"
    } else {
        asset_path
    };

    if let Some(resolver) = &server.asset_resolver {
        if let Some(asset) = resolver.get(asset_path.to_owned()) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, asset.mime_type)
                .body(Body::from(asset.bytes))
                .unwrap();
        }

        if asset_path != "index.html" {
            if let Some(index) = resolver.get("index.html".to_owned()) {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, index.mime_type)
                    .body(Body::from(index.bytes))
                    .unwrap();
            }
        }
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 Not Found"))
        .unwrap()
}
