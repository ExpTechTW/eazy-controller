use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioSession {
    pub name: String,
    pub volume: f32,
    pub is_muted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaInfo {
    pub session_id: String,
    pub app_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub is_playing: bool,
    pub thumbnail: Option<String>,
    pub can_go_next: bool,
    pub can_go_previous: bool,
}
