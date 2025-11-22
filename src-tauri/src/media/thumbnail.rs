#[cfg(target_os = "windows")]
use crate::utils::ComGuard;

#[cfg(target_os = "windows")]
pub fn get_thumbnail_safe(
    props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::time::{Duration, Instant};
    use windows::Storage::Streams::{Buffer, DataReader};

    let _com_guard = ComGuard::new();

    let thumbnail_ref = match props.Thumbnail() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let stream_future = match thumbnail_ref.OpenReadAsync() {
        Ok(f) => f,
        Err(_) => return None,
    };

    let start = Instant::now();
    let timeout = Duration::from_secs(2);

    let stream = match stream_future.get() {
        Ok(s) => {
            if start.elapsed() > timeout {
                return None;
            }
            s
        }
        Err(_) => return None,
    };

    let size = match stream.Size() {
        Ok(s) => s,
        Err(_) => return None,
    };

    if size == 0 || size > 5_000_000 {
        return None;
    }

    let buffer = match Buffer::Create(size as u32) {
        Ok(b) => b,
        Err(_) => return None,
    };

    let read_future = match stream.ReadAsync(
        &buffer,
        size as u32,
        windows::Storage::Streams::InputStreamOptions::None,
    ) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let buffer_read = match read_future.get() {
        Ok(b) => {
            if start.elapsed() > timeout {
                return None;
            }
            b
        }
        Err(_) => return None,
    };

    let reader = match DataReader::FromBuffer(&buffer_read) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let length = match reader.UnconsumedBufferLength() {
        Ok(l) => l,
        Err(_) => return None,
    };

    if length == 0 || length > 5_000_000 {
        return None;
    }

    let mut bytes = vec![0u8; length as usize];
    if reader.ReadBytes(&mut bytes).is_err() {
        return None;
    }

    Some(general_purpose::STANDARD.encode(&bytes))
}
