use crate::models::AudioSession;
use std::collections::HashMap;

#[tauri::command]
pub fn get_audio_sessions() -> Result<Vec<AudioSession>, String> {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use winmix::WinMix;

        unsafe {
            let winmix = WinMix::default();
            let sessions_result = winmix
                .enumerate()
                .map_err(|e| format!("無法獲取音樂資料清單: {:?}", e))?;

            let mut sessions_map: HashMap<String, AudioSession> = HashMap::new();

            for session in sessions_result {
                let name = if !session.path.is_empty() {
                    Path::new(&session.path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&session.path)
                        .to_string()
                } else {
                    format!("PID {}", session.pid)
                };

                let volume = session.vol.get_master_volume().unwrap_or(0.0);
                let is_muted = session.vol.get_mute().unwrap_or(false);

                sessions_map.entry(name.clone()).or_insert(AudioSession {
                    name,
                    volume,
                    is_muted,
                });
            }

            let audio_sessions: Vec<AudioSession> = sessions_map.into_values().collect();

            Ok(audio_sessions)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 設定指定應用程式的音量
/// @param session_name 應用程式名稱
/// @param volume 音量大小 (0.0 ~ 1.0)
#[tauri::command]
pub fn set_session_volume(session_name: String, volume: f32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use winmix::WinMix;

        unsafe {
            let winmix = WinMix::default();
            let sessions = winmix
                .enumerate()
                .map_err(|e| format!("無法獲取音樂資料清單: {:?}", e))?;

            let mut found = false;
            for session in sessions {
                let name = if !session.path.is_empty() {
                    Path::new(&session.path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&session.path)
                        .to_string()
                } else {
                    format!("PID {}", session.pid)
                };

                if name == session_name {
                    session
                        .vol
                        .set_master_volume(volume)
                        .map_err(|e| format!("無法設定音量: {:?}", e))?;
                    found = true;
                }
            }

            if found {
                Ok(())
            } else {
                Err(format!("找不到: '{}'", session_name))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 設定指定應用程式的靜音狀態
/// @param session_name 應用程式名稱
/// @param mute 是否靜音 (true=靜音, false=取消靜音)
#[tauri::command]
pub fn set_session_mute(session_name: String, mute: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use winmix::WinMix;

        unsafe {
            let winmix = WinMix::default();
            let sessions = winmix
                .enumerate()
                .map_err(|e| format!("無法獲取音樂資料清單: {:?}", e))?;

            let mut found = false;
            for session in sessions {
                let name = if !session.path.is_empty() {
                    Path::new(&session.path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&session.path)
                        .to_string()
                } else {
                    format!("PID {}", session.pid)
                };

                if name == session_name {
                    session
                        .vol
                        .set_mute(mute)
                        .map_err(|e| format!("無法設定靜音: {:?}", e))?;
                    found = true;
                }
            }

            if found {
                Ok(())
            } else {
                Err(format!("找不到: '{}'", session_name))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}
