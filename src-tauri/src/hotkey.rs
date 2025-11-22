/// 儲存全域快捷鍵設定到本地
#[tauri::command]
pub fn save_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("無法打開儲存: {:?}", e))?;

    store.set("hotkey", serde_json::json!(hotkey));

    store.save().map_err(|e| format!("無法保存設定: {:?}", e))?;

    Ok(())
}

/// 載入全域快捷鍵設定
#[tauri::command]
pub fn load_hotkey(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("無法打開儲存: {:?}", e))?;

    let hotkey = store
        .get("hotkey")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "Alt+Z".to_string());

    Ok(hotkey)
}

/// 取消註冊所有全域快捷鍵
#[tauri::command]
pub fn unregister_all_hotkeys(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // 清除所有快捷鍵
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("無法取消註冊快捷鍵: {:?}", e))?;

    Ok(())
}

/// 註冊全域快捷鍵
#[tauri::command]
pub fn register_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    use tauri::Manager;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let _ = app.global_shortcut().unregister_all();

    if hotkey.is_empty() {
        return Ok(());
    }

    let shortcut: Shortcut = hotkey
        .parse()
        .map_err(|e| format!("無法解析快捷鍵: {:?}", e))?;

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let is_minimized = window.is_minimized().unwrap_or(false);
                    let is_visible = window.is_visible().unwrap_or(false);
                    if is_minimized {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    } else if is_visible {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .map_err(|e| format!("無法註冊快捷鍵: {:?}", e))?;

    Ok(())
}
