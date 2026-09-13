//! Concurrent, independently bounded pipe collection for real process subjects.
use std::io::Read;
use std::thread::JoinHandle;

pub(super) const MAXIMUM_PIPE_BYTES: usize = 4 * 1024 * 1024;
type Capture = JoinHandle<Result<Vec<u8>, String>>;

pub(super) fn start(
    pipe: Option<impl Read + Send + 'static>,
    label: &'static str,
) -> Option<Capture> {
    pipe.map(|pipe| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            pipe.take(MAXIMUM_PIPE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| format!("read guarded Phase 8 {label}: {error}"))?;
            if bytes.len() > MAXIMUM_PIPE_BYTES {
                return Err(format!(
                    "Phase 8 {label} exceeded {MAXIMUM_PIPE_BYTES}-byte capture bound"
                ));
            }
            Ok(bytes)
        })
    })
}

pub(super) fn finish(capture: Option<Capture>) -> Result<Vec<u8>, String> {
    match capture {
        None => Ok(Vec::new()),
        Some(capture) => capture
            .join()
            .map_err(|_| "Phase 8 output collector panicked".to_owned())?,
    }
}
