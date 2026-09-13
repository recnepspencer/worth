use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn write_runtime_identity(path: &Path, identity: String) -> Result<(), String> {
    std::fs::write(path.with_extension("runtime"), identity)
        .map_err(|error| format!("write C8 writer runtime identity: {error}"))
}

pub(super) fn write_ready(path: &Path, error_context: &str) -> Result<(), String> {
    std::fs::write(path.with_extension("ready"), b"ready")
        .map_err(|error| format!("{error_context}: {error}"))
}

pub(super) fn write_reached(
    path: &Path,
    contents: &[u8],
    error_context: &str,
) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|error| format!("{error_context}: {error}"))
}

pub(super) fn wait_for_parent(marker: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !marker.exists() {
        if Instant::now() >= deadline {
            panic!("C8 writer start marker timeout: {}", marker.display());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn park_forever() -> ! {
    loop {
        std::thread::park();
    }
}
