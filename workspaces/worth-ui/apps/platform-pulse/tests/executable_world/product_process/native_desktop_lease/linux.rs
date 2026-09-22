//! Linux: the kernel's advisory lock is the lease. `File::try_lock` is
//! `flock(LOCK_EX | LOCK_NB)`, held by the open file description and dropped
//! by the kernel the moment its last descriptor closes, however the owner
//! ends. That is the same posture as Windows' share mode: no pid liveness
//! heuristic, no stale state, no reclaim step in which two contenders could
//! both decide they won. The file's content names the owner for diagnosis
//! only; it decides nothing. The desktop is the X display, so two displays
//! hold two independent leases.
//!
//! The owner unlinks its lock while still holding it, so a contender that
//! opened the old inode before the unlink can lock an orphan. Ownership is
//! therefore the lock *and* the locked inode still being the one at `path`;
//! anything else is contention and the poll tries again on a fresh open.
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::LeaseAttempt;

const LEASE_PREFIX: &str = "worth-ui-native-desktop-";
const LEASE_SUFFIX: &str = "-v1.lock";

pub(super) fn lease_path() -> PathBuf {
    let root = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    root.join(lease_file_name(std::env::var("DISPLAY").ok().as_deref()))
}

pub(super) fn attempt(path: &Path) -> LeaseAttempt {
    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
    {
        Ok(file) => file,
        Err(error) => return LeaseAttempt::Failed(error),
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return LeaseAttempt::Contended,
        Err(TryLockError::Error(error)) => return LeaseAttempt::Failed(error),
    }
    match locked_inode_is_at_path(&file, path) {
        Ok(true) => {}
        Ok(false) => return LeaseAttempt::Contended,
        Err(error) => return LeaseAttempt::Failed(error),
    }
    match name_owner(&file) {
        Ok(()) => LeaseAttempt::Owned(file),
        Err(error) => LeaseAttempt::Failed(error),
    }
}

/// The owner removes the lock file before its lock is released, so the file
/// a later contender creates at `path` is a fresh inode nobody else holds.
pub(super) fn release(file: File, path: &Path) {
    release_with_contention_between(file, path, || {});
}

/// The two release steps with a seam between them. In the opposite order a
/// contender arriving between the steps locks the inode still at `path`,
/// becomes its owner, and then has that file unlinked from under it, so the
/// next contender owns a second inode at the same time. `between` is where
/// the falsifier plants that contender; production passes nothing.
fn release_with_contention_between(file: File, path: &Path, between: impl FnOnce()) {
    let _ = fs::remove_file(path);
    between();
    drop(file);
}

fn locked_inode_is_at_path(file: &File, path: &Path) -> std::io::Result<bool> {
    let locked = file.metadata()?;
    match fs::metadata(path) {
        Ok(current) => Ok(current.dev() == locked.dev() && current.ino() == locked.ino()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn name_owner(mut file: &File) -> std::io::Result<()> {
    file.set_len(0)?;
    write!(file, "{}", std::process::id())?;
    file.sync_all()
}

/// Diagnostic only: whose pid the lock file names.
#[cfg(test)]
fn holder_process_id(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn lease_file_name(display: Option<&str>) -> String {
    let display = match display {
        Some(display) if !display.is_empty() => display
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                }
            })
            .collect(),
        _ => String::from("unset"),
    };
    format!("{LEASE_PREFIX}{display}{LEASE_SUFFIX}")
}

#[cfg(test)]
mod tests;
