#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

pub(crate) use debug_log;

#[cfg(target_os = "windows")]
pub struct ComGuard;

#[cfg(target_os = "windows")]
impl ComGuard {
    pub fn new() -> Self {
        use windows::Win32::System::Com::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        ComGuard
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        use windows::Win32::System::Com::CoUninitialize;
        unsafe {
            CoUninitialize();
        }
    }
}
