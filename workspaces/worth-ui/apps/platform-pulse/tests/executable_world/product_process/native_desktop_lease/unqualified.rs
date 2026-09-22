//! A target with no qualified native desktop (milestone-3.10.3 §D5 compile-
//! only posture). No product process launches here, so an attempt to lease
//! the desktop is a typed failure rather than a mechanism this target lacks.
use std::path::{Path, PathBuf};

use super::LeaseAttempt;

const LEASE_FILE: &str = "worth-ui-native-desktop-v1.lock";

pub(super) fn lease_path() -> PathBuf {
    std::env::temp_dir().join(LEASE_FILE)
}

pub(super) fn attempt(_path: &Path) -> LeaseAttempt {
    LeaseAttempt::Failed(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no qualified native desktop on this target",
    ))
}

/// Never reached: no lease is ever owned here.
pub(super) fn release(file: std::fs::File, path: &Path) {
    drop(file);
    let _ = std::fs::remove_file(path);
}
