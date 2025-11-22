use crate::media::cache::{MEDIA_SESSION_CACHE, SESSION_MANAGER_CACHE};
use crate::media::thumbnail::get_thumbnail_safe;
use crate::models::MediaInfo;
use std::time::Duration;

#[cfg(target_os = "windows")]
use crate::utils::ComGuard;

/// 獲取所有媒體會話(播放器)的資訊列表
#[tauri::command]
pub fn get_all_media_sessions() -> Result<Vec<MediaInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        use std::thread;
        use windows::Media::Control::*;

        let cache_max_age = Duration::from_millis(500);

        if !MEDIA_SESSION_CACHE.should_update(cache_max_age) {
            return Ok(MEDIA_SESSION_CACHE.get());
        }

        if !MEDIA_SESSION_CACHE.start_update() {
            return Ok(MEDIA_SESSION_CACHE.get());
        }

        thread::spawn(move || {
            let _com_guard = ComGuard::new();

            let session_manager = match SESSION_MANAGER_CACHE.get_or_refresh(Duration::from_secs(2))
            {
                Some(sm) => sm,
                None => {
                    MEDIA_SESSION_CACHE.finish_update(Vec::new());
                    return;
                }
            };

            let sessions = match session_manager.GetSessions() {
                Ok(s) => s,
                Err(_) => {
                    MEDIA_SESSION_CACHE.finish_update(Vec::new());
                    return;
                }
            };

            let mut media_infos = Vec::new();
            let session_count = sessions.Size().unwrap_or(0);

            for i in 0..session_count {
                if let Ok(session) = sessions.GetAt(i) {
                    if session.SourceAppUserModelId().is_err() {
                        continue;
                    }

                    let app_name = session
                        .SourceAppUserModelId()
                        .unwrap_or_default()
                        .to_string();

                    if let Ok(props_async) = session.TryGetMediaPropertiesAsync() {
                        if let Ok(props) = props_async.get() {
                            let title = props.Title().unwrap_or_default().to_string();
                            let artist = props.Artist().unwrap_or_default().to_string();
                            let album = props.AlbumTitle().unwrap_or_default().to_string();

                            let playback_info = session.GetPlaybackInfo().ok();
                            let is_playing = playback_info
                            .as_ref()
                            .and_then(|info| info.PlaybackStatus().ok())
                            .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                            .unwrap_or(false);

                            let controls =
                                playback_info.as_ref().and_then(|info| info.Controls().ok());

                            let next_result =
                                controls.as_ref().and_then(|c| c.IsNextEnabled().ok());
                            let prev_result =
                                controls.as_ref().and_then(|c| c.IsPreviousEnabled().ok());

                            let can_go_next = next_result.unwrap_or(true);
                            let can_go_previous = prev_result.unwrap_or(true);

                            let session_id = format!("{}_{}", app_name, i);

                            media_infos.push(MediaInfo {
                                session_id,
                                app_name,
                                title,
                                artist,
                                album,
                                is_playing,
                                thumbnail: None,
                                can_go_next,
                                can_go_previous,
                            });
                        }
                    }
                }
            }

            MEDIA_SESSION_CACHE.finish_update(media_infos);
        });

        Ok(MEDIA_SESSION_CACHE.get())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 獲取當前正在播放的媒體資訊(第一個)
/// 返回正在播放的媒體資訊，如果沒有則返回 None
#[tauri::command]
pub fn get_media_info() -> Result<Option<MediaInfo>, String> {
    let all_sessions = get_all_media_sessions()?;
    Ok(all_sessions.into_iter().next())
}

#[cfg(target_os = "windows")]
pub fn get_session_by_id(
    session_id: &str,
) -> Result<windows::Media::Control::GlobalSystemMediaTransportControlsSession, String> {
    let session_manager = SESSION_MANAGER_CACHE
        .get_or_refresh(Duration::from_secs(2))
        .ok_or("無法取得訊息管理器".to_string())?;

    let sessions = session_manager
        .GetSessions()
        .map_err(|e| format!("無法取得會話列表: {:?}", e))?;

    let session_count = sessions.Size().unwrap_or(0);

    for i in 0..session_count {
        if let Ok(session) = sessions.GetAt(i) {
            if let Ok(app_name) = session.SourceAppUserModelId() {
                let current_session_id = format!("{}_{}", app_name.to_string(), i);
                if current_session_id == session_id {
                    return Ok(session);
                }
            }
        }
    }

    Err(format!("找不到會話: {}", session_id))
}

/// 專輯封面
/// @param session_id
/// 返回 Base64 編碼的圖片字串
#[tauri::command]
pub fn get_media_thumbnail(session_id: Option<String>) -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Media::Control::*;

        let session = if let Some(id) = session_id {
            get_session_by_id(&id)?
        } else {
            let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .map_err(|e| format!("無法取得訊息管理器: {:?}", e))?
                .get()
                .map_err(|e| format!("無法取得: {:?}", e))?;

            session_manager
                .GetCurrentSession()
                .map_err(|e| format!("無法取得: {:?}", e))?
        };

        if session.SourceAppUserModelId().is_err() {
            return Ok(None);
        }

        let props = session
            .TryGetMediaPropertiesAsync()
            .map_err(|e| format!("無法取得媒體屬性: {:?}", e))?
            .get()
            .map_err(|e| format!("無法取得媒體屬性: {:?}", e))?;

        Ok(get_thumbnail_safe(&props))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 切換媒體播放/暫停狀態
/// @param session_id
/// 如果正在播放則暫停，如果已暫停則播放
#[tauri::command]
pub fn media_play_pause(session_id: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Media::Control::*;

        tauri::async_runtime::spawn_blocking(move || {
            let _com_guard = ComGuard::new();

            let session = if let Some(id) = session_id {
                match get_session_by_id(&id) {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            } else {
                let session_manager =
                    match SESSION_MANAGER_CACHE.get_or_refresh(Duration::from_secs(2)) {
                        Some(sm) => sm,
                        None => {
                            return;
                        }
                    };

                match session_manager.GetCurrentSession() {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            };

            if session.SourceAppUserModelId().is_err() {
                return;
            }

            let playback_info = match session.GetPlaybackInfo() {
                Ok(info) => info,
                Err(_) => {
                    return;
                }
            };

            let status = match playback_info.PlaybackStatus() {
                Ok(s) => s,
                Err(_) => {
                    return;
                }
            };

            if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                if let Ok(async_op) = session.TryPauseAsync() {
                    let _ = async_op.get();
                }
            } else {
                if let Ok(async_op) = session.TryPlayAsync() {
                    let _ = async_op.get();
                }
            }
        });

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 跳到下一首歌曲/媒體
/// @param session_id
#[tauri::command]
pub fn media_next(session_id: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let _com_guard = ComGuard::new();

            let session = if let Some(id) = session_id {
                match get_session_by_id(&id) {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            } else {
                let session_manager =
                    match SESSION_MANAGER_CACHE.get_or_refresh(Duration::from_secs(2)) {
                        Some(sm) => sm,
                        None => {
                            return;
                        }
                    };

                match session_manager.GetCurrentSession() {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            };

            if session.SourceAppUserModelId().is_err() {
                return;
            }

            if let Ok(async_op) = session.TrySkipNextAsync() {
                let _ = async_op.get();
            }
        });

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 回到上一首歌曲/媒體
/// @param session_id ID
#[tauri::command]
pub fn media_previous(session_id: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let _com_guard = ComGuard::new();

            let session = if let Some(id) = session_id {
                match get_session_by_id(&id) {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            } else {
                let session_manager =
                    match SESSION_MANAGER_CACHE.get_or_refresh(Duration::from_secs(2)) {
                        Some(sm) => sm,
                        None => {
                            return;
                        }
                    };

                match session_manager.GetCurrentSession() {
                    Ok(s) => s,
                    Err(_) => {
                        return;
                    }
                }
            };

            if session.SourceAppUserModelId().is_err() {
                return;
            }

            if let Ok(async_op) = session.TrySkipPreviousAsync() {
                let _ = async_op.get();
            }
        });

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}
