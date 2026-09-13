use std::path::Path;
use std::time::{Duration, Instant};

use super::super::child_lifecycle::ProcessChildGuard;

pub(crate) fn c8_writer_binary_path() -> std::path::PathBuf {
    super::super::support_binaries::phase_eight_process_binaries()
        .writer()
        .path()
        .to_path_buf()
}

pub(super) fn wait_for_marker(
    child: &mut ProcessChildGuard,
    marker: &Path,
    label: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(120);
    while !marker.exists() {
        if Instant::now() >= deadline {
            return Err(format!("{label} marker timeout: {}", marker.display()));
        }
        if child
            .child_mut()
            .try_wait()
            .map_err(|error| format!("poll {label} child: {error}"))?
            .is_some()
        {
            return Err(format!("production writer exited before {label}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}
