use serde_json::{json, Value};

pub async fn handle_message(msg: Value) -> Option<Value> {
    let msg_type = msg.get("type")?.as_str()?;

    match msg_type {
        "get_audio_sessions" => handle_get_audio_sessions().await,
        "set_session_volume" => handle_set_session_volume(msg).await,
        "set_session_mute" => handle_set_session_mute(msg).await,
        "get_audio_devices" => handle_get_audio_devices().await,
        "set_default_device" => handle_set_default_device(msg).await,
        "get_default_device_volume" => handle_get_default_device_volume().await,
        "set_default_device_volume" => handle_set_default_device_volume(msg).await,
        "get_default_device_mute" => handle_get_default_device_mute().await,
        "set_default_device_mute" => handle_set_default_device_mute(msg).await,
        "get_all_media_sessions" => handle_get_all_media_sessions().await,
        "get_media_info" => handle_get_media_info().await,
        "get_media_thumbnail" => handle_get_media_thumbnail(msg).await,
        "media_play_pause" => handle_media_play_pause(msg),
        "media_next" => handle_media_next(msg),
        "media_previous" => handle_media_previous(msg),
        _ => Some(json!({
            "type": "error",
            "message": format!("未知的消息類型: {}", msg_type)
        })),
    }
}

// === Audio Sessions ===

async fn handle_get_audio_sessions() -> Option<Value> {
    match crate::get_audio_sessions() {
        Ok(sessions) => Some(json!({
            "type": "audio_sessions",
            "data": sessions
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_set_session_volume(msg: Value) -> Option<Value> {
    let data = msg.get("data")?;
    let session_name = data.get("session_name")?.as_str()?.to_string();
    let volume = data.get("volume")?.as_f64()? as f32;

    match crate::set_session_volume(session_name, volume) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "音量設定成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_set_session_mute(msg: Value) -> Option<Value> {
    let data = msg.get("data")?;
    let session_name = data.get("session_name")?.as_str()?.to_string();
    let mute = data.get("mute")?.as_bool()?;

    match crate::set_session_mute(session_name, mute) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "靜音設定成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

// === Audio Devices ===

async fn handle_get_audio_devices() -> Option<Value> {
    match crate::get_audio_devices() {
        Ok(devices) => Some(json!({
            "type": "audio_devices",
            "data": devices
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_set_default_device(msg: Value) -> Option<Value> {
    let data = msg.get("data")?;
    let device_id = data.get("device_id")?.as_str()?.to_string();

    match crate::set_default_device(device_id) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "預設裝置設定成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_get_default_device_volume() -> Option<Value> {
    match crate::get_default_device_volume() {
        Ok(volume) => Some(json!({
            "type": "default_device_volume",
            "data": volume
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_set_default_device_volume(msg: Value) -> Option<Value> {
    let data = msg.get("data")?;
    let volume = data.get("volume")?.as_f64()? as f32;

    match crate::set_default_device_volume(volume) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "預設裝置音量設定成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_get_default_device_mute() -> Option<Value> {
    match crate::get_default_device_mute() {
        Ok(muted) => Some(json!({
            "type": "default_device_mute",
            "data": muted
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_set_default_device_mute(msg: Value) -> Option<Value> {
    let data = msg.get("data")?;
    let mute = data.get("mute")?.as_bool()?;

    match crate::set_default_device_mute(mute) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "預設裝置靜音設定成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

// === Media Control ===

async fn handle_get_all_media_sessions() -> Option<Value> {
    match crate::get_all_media_sessions() {
        Ok(sessions) => Some(json!({
            "type": "all_media_sessions",
            "data": sessions
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_get_media_info() -> Option<Value> {
    match crate::get_media_info() {
        Ok(info) => Some(json!({
            "type": "media_info",
            "data": info
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

async fn handle_get_media_thumbnail(msg: Value) -> Option<Value> {
    let data = msg.get("data");
    let session_id = data.and_then(|d| d.get("session_id")).and_then(|s| s.as_str()).map(|s| s.to_string());

    match crate::get_media_thumbnail(session_id) {
        Ok(thumbnail) => Some(json!({
            "type": "media_thumbnail",
            "data": thumbnail
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

fn handle_media_play_pause(msg: Value) -> Option<Value> {
    let data = msg.get("data");
    let session_id = data.and_then(|d| d.get("session_id")).and_then(|s| s.as_str()).map(|s| s.to_string());

    match crate::media_play_pause(session_id) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "播放/暫停成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

fn handle_media_next(msg: Value) -> Option<Value> {
    let data = msg.get("data");
    let session_id = data.and_then(|d| d.get("session_id")).and_then(|s| s.as_str()).map(|s| s.to_string());

    match crate::media_next(session_id) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "下一首成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}

fn handle_media_previous(msg: Value) -> Option<Value> {
    let data = msg.get("data");
    let session_id = data.and_then(|d| d.get("session_id")).and_then(|s| s.as_str()).map(|s| s.to_string());

    match crate::media_previous(session_id) {
        Ok(_) => Some(json!({
            "type": "success",
            "message": "上一首成功"
        })),
        Err(e) => Some(json!({
            "type": "error",
            "message": e
        })),
    }
}
