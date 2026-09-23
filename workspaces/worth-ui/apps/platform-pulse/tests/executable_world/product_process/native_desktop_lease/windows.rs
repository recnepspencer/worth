//! Windows: the kernel's share mode is the lock. An exclusive open fails with
//! a sharing violation while any other process holds the file, and the file
//! is released the moment that process dies, so there is no stale state.
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use super::LeaseAttempt;

const LEASE_FILE: &str = "worth-ui-native-desktop-v1.lock";

pub(super) fn lease_path() -> PathBuf {
    std::env::temp_dir().join(LEASE_FILE)
}

pub(super) fn attempt(path: &Path) -> LeaseAttempt {
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(path)
    {
        Ok(file) => LeaseAttempt::Owned(file),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::PermissionDenied | ErrorKind::WouldBlock
            ) || matches!(error.raw_os_error(), Some(32 | 33)) =>
        {
            LeaseAttempt::Contended
        }
        Err(error) => LeaseAttempt::Failed(error),
    }
}

/// The exclusive handle must close before the file can go: without
/// `FILE_SHARE_DELETE` the owner's own open forbids the delete. A contender
/// that opens in the gap simply keeps the file, which the delete then leaves.
pub(super) fn release(file: File, path: &Path) {
    drop(file);
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::LEASE_FILE;

    #[test]
    fn the_windows_lease_lives_in_the_temp_directory() {
        assert_eq!(super::lease_path(), std::env::temp_dir().join(LEASE_FILE));
    }
}
