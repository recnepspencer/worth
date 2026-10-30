pub(crate) use std::path::Path;
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard;

pub(crate) fn c8_writer_binary_path() -> std::path::PathBuf {
    crate::support_binaries::phase_eight_process_binaries()
        .writer()
        .path()
        .to_path_buf()
}

pub(crate) fn wait_for_marker(
    child: &mut ProcessChildGuard,
    marker: &Path,
    label: &str,
) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !marker.exists() {
        if std::time::Instant::now() >= deadline {
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
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    Ok(())
}
