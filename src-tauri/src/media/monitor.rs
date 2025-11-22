use crate::media::controls::{get_all_media_sessions, get_media_thumbnail};
use crate::utils::debug_log;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

/// 媒體監聽循環
pub fn media_monitor_loop<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    http_server: Arc<crate::http_server::HttpServer<R>>,
) {
    let mut last_all_sessions: Option<String> = None;

    loop {
        thread::sleep(Duration::from_millis(500));

        match get_all_media_sessions() {
            Ok(all_sessions) => {
                if let Ok(current_json) = serde_json::to_string(&all_sessions) {
                    let has_changed = last_all_sessions.as_ref() != Some(&current_json);

                    if has_changed {
                        last_all_sessions = Some(current_json);

                        for media_info in &all_sessions {
                            let _ = app_handle.emit("media-info-updated", media_info);

                            let ws_message = serde_json::json!({
                                "type": "media_info_updated",
                                "data": media_info
                            });
                            http_server.broadcast(ws_message.to_string());
                        }

                        if let Some(first_session) = all_sessions.first() {
                            let is_browser =
                                first_session.app_name.to_lowercase().contains("chrome")
                                    || first_session.app_name.to_lowercase().contains("edge")
                                    || first_session.app_name.to_lowercase().contains("firefox")
                                    || first_session.app_name.to_lowercase().contains("opera")
                                    || first_session.app_name.to_lowercase().contains("brave");

                            if is_browser {
                                let app_handle_clone = app_handle.clone();
                                let http_server_clone = Arc::clone(&http_server);
                                let session_id = first_session.session_id.clone();
                                thread::spawn(move || {
                                    if let Ok(Some(thumbnail)) =
                                        get_media_thumbnail(Some(session_id))
                                    {
                                        let _ = app_handle_clone
                                            .emit("media-thumbnail-updated", &thumbnail);

                                        let ws_message = serde_json::json!({
                                            "type": "media_thumbnail_updated",
                                            "data": thumbnail
                                        });
                                        http_server_clone.broadcast(ws_message.to_string());
                                    }
                                });
                            }
                        }
                    }
                }

                if all_sessions.is_empty() && last_all_sessions.is_some() {
                    last_all_sessions = None;
                    let _ = app_handle.emit("media-info-cleared", ());

                    let ws_message = serde_json::json!({
                        "type": "media_info_cleared"
                    });
                    http_server.broadcast(ws_message.to_string());
                }
            }
            Err(_e) => {
                debug_log!("監聽器錯誤: {}", _e);
            }
        }
    }
}
