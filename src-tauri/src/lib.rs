mod audio;
mod hotkey;
mod http_server;
mod media;
mod message_handler;
mod models;
mod utils;

use audio::*;
use hotkey::*;
use media::*;
use std::sync::Arc;
use tauri_plugin_updater::UpdaterExt;

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("{downloaded} / {content_length:?}");
                },
                || {
                    println!("downloaded");
                },
            )
            .await?;

        println!("installed");
        app.restart();
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder};
    use tauri::Manager;

    tauri::Builder::default()
        .plugin({
            use tauri_plugin_prevent_default::{Builder, KeyboardShortcut, ModifierKey};
            Builder::new()
                .shortcut(KeyboardShortcut::new("F5"))
                .shortcut(KeyboardShortcut::with_modifiers(
                    "r",
                    &[ModifierKey::CtrlKey],
                ))
                .build()
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            let show_item = MenuItem::with_id(app, "show", "開啟", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "關閉", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("eazy-controller")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button,
                        button_state,
                        ..
                    } = event
                    {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            let asset_resolver = app.asset_resolver();
            let http_server =
                Arc::new(http_server::HttpServer::new().with_asset_resolver(asset_resolver));
            let http_server_clone = Arc::clone(&http_server);

            tauri::async_runtime::spawn(async move {
                http_server_clone.start(8800).await;
            });

            let app_handle = app.handle().clone();
            let http_server_monitor = Arc::clone(&http_server);
            std::thread::spawn(move || {
                media_monitor_loop(app_handle, http_server_monitor);
            });

            let app_handle = app.handle().clone();
            let _ = register_hotkey(app_handle, "Alt+Z".to_string());

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = update(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_sessions,
            set_session_volume,
            set_session_mute,
            get_audio_devices,
            set_default_device,
            get_default_device_volume,
            set_default_device_volume,
            get_default_device_mute,
            set_default_device_mute,
            get_media_info,
            get_all_media_sessions,
            get_media_thumbnail,
            media_play_pause,
            media_next,
            media_previous,
            register_hotkey,
            unregister_all_hotkeys,
            save_hotkey,
            load_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("無法運行 Tauri");
}
