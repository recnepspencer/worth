use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::windows::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const ACQUISITION_POLL_INTERVAL: Duration = Duration::from_millis(10);
const LEASE_FILE: &str = "worth-ui-native-desktop-v1.lock";
static IN_PROCESS_NATIVE_DESKTOP: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) struct NativeDesktopCourtroomLease {
    _guard: MutexGuard<'static, ()>,
}

pub(super) struct NativeDesktopLease {
    file: Option<File>,
    path: PathBuf,
    owner_process_id: u32,
}

impl NativeDesktopCourtroomLease {
    pub(super) fn acquire() -> Self {
        let courtroom = IN_PROCESS_NATIVE_DESKTOP.get_or_init(|| Mutex::new(()));
        let guard = courtroom
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self { _guard: guard }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeDesktopLeaseDeadline;

impl NativeDesktopLease {
    pub(super) fn acquire(deadline: Instant) -> Result<Self, NativeDesktopLeaseDeadline> {
        let path = std::env::temp_dir().join(LEASE_FILE);
        loop {
            match open_exclusive(&path) {
                Ok(file) => {
                    return Ok(Self {
                        file: Some(file),
                        path,
                        owner_process_id: std::process::id(),
                    });
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::PermissionDenied | ErrorKind::WouldBlock
                    ) && Instant::now() < deadline =>
                {
                    thread::sleep(ACQUISITION_POLL_INTERVAL);
                }
                Err(_) => return Err(NativeDesktopLeaseDeadline),
            }
        }
    }

    pub(super) fn owner_process_id(&self) -> u32 {
        self.owner_process_id
    }
}

impl Drop for NativeDesktopLease {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

fn open_exclusive(path: &PathBuf) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(path)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{NativeDesktopCourtroomLease, NativeDesktopLease};

    #[test]
    fn exclusive_desktop_lease_rejects_a_concurrent_owner() {
        const CHILD_MARKER: &str = "WORTH_UI_DESKTOP_LEASE_CHILD";
        if std::env::var_os(CHILD_MARKER).is_some() {
            assert!(
                NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(20)).is_err()
            );
            return;
        }
        let _courtroom = NativeDesktopCourtroomLease::acquire();
        let first = NativeDesktopLease::acquire(Instant::now() + Duration::from_secs(1))
            .expect("first cross-process lease");
        assert_eq!(first.owner_process_id(), std::process::id());
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .env(CHILD_MARKER, "1")
            .args([
                "product_process::native_desktop_lease::tests::exclusive_desktop_lease_rejects_a_concurrent_owner",
                "--exact",
                "--nocapture",
            ])
            .output()
            .expect("launch cross-process lease contender");
        assert!(
            child.status.success(),
            "child lease contender failed: {}",
            String::from_utf8_lossy(&child.stderr)
        );
        let child_stdout = String::from_utf8_lossy(&child.stdout);
        assert!(
            child_stdout.contains("running 1 test") && child_stdout.contains("1 passed"),
            "child lease contender did not execute exactly one test: {child_stdout}"
        );
        drop(first);
        NativeDesktopLease::acquire(Instant::now() + Duration::from_secs(1))
            .expect("released lease can be reacquired");
    }
}
