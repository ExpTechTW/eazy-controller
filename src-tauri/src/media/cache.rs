use crate::models::MediaInfo;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub struct MediaSessionCache {
    data: RwLock<Vec<MediaInfo>>,
    last_update: RwLock<Option<Instant>>,
    is_updating: AtomicBool,
}

impl MediaSessionCache {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(Vec::new()),
            last_update: RwLock::new(None),
            is_updating: AtomicBool::new(false),
        }
    }

    pub fn should_update(&self, max_age: Duration) -> bool {
        let last = self.last_update.read().unwrap();
        match *last {
            None => true,
            Some(time) => time.elapsed() > max_age,
        }
    }

    pub fn start_update(&self) -> bool {
        self.is_updating.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    pub fn finish_update(&self, data: Vec<MediaInfo>) {
        *self.data.write().unwrap() = data;
        *self.last_update.write().unwrap() = Some(Instant::now());
        self.is_updating.store(false, Ordering::SeqCst);
    }

    pub fn get(&self) -> Vec<MediaInfo> {
        self.data.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn find_by_id(&self, session_id: &str) -> Option<MediaInfo> {
        let data = self.data.read().unwrap();
        data.iter().find(|s| s.session_id == session_id).cloned()
    }
}

#[cfg(target_os = "windows")]
pub struct SessionManagerCache {
    manager: RwLock<Option<windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager>>,
    last_update: RwLock<Option<Instant>>,
}

#[cfg(target_os = "windows")]
impl SessionManagerCache {
    pub fn new() -> Self {
        Self {
            manager: RwLock::new(None),
            last_update: RwLock::new(None),
        }
    }

    pub fn get_or_refresh(&self, max_age: Duration) -> Option<windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager> {
        let should_refresh = {
            let last = self.last_update.read().unwrap();
            match *last {
                None => true,
                Some(time) => time.elapsed() > max_age,
            }
        };

        if should_refresh {
            use windows::Media::Control::*;
            match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
                Ok(async_op) => match async_op.get() {
                    Ok(new_manager) => {
                        *self.manager.write().unwrap() = Some(new_manager.clone());
                        *self.last_update.write().unwrap() = Some(Instant::now());
                        return Some(new_manager);
                    },
                    Err(_) => {}
                }
                Err(_) => {}
            }
        }

        self.manager.read().unwrap().clone()
    }
}

lazy_static::lazy_static! {
    pub static ref MEDIA_SESSION_CACHE: MediaSessionCache = MediaSessionCache::new();
}

#[cfg(target_os = "windows")]
lazy_static::lazy_static! {
    pub static ref SESSION_MANAGER_CACHE: SessionManagerCache = SessionManagerCache::new();
}
