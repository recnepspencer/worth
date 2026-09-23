//! One product process per native desktop. Two courtroom runners on the same
//! desktop would each observe the other's window; the lease types that
//! contention as an environment denial before any product effect. The
//! cross-process mechanism is the platform's own kernel-arbitrated exclusive
//! hold (share mode on Windows, `flock` on Linux), released by the kernel
//! when the owner dies: `windows.rs`, `linux.rs` and `unqualified.rs` each
//! expose `lease_path`, `attempt` and `release`, the last because the order
//! of close and unlink differs per kernel.
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod unqualified;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
use unqualified as platform;
#[cfg(target_os = "windows")]
use windows as platform;

const ACQUISITION_POLL_INTERVAL: Duration = Duration::from_millis(10);
static IN_PROCESS_NATIVE_DESKTOP: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) struct NativeDesktopCourtroomLease {
    _guard: MutexGuard<'static, ()>,
}

pub(super) struct NativeDesktopLease {
    file: Option<File>,
    path: PathBuf,
    owner_process_id: u32,
}

/// One attempt to take the lease at a path, in the vocabulary every platform
/// shares: the acquire loop polls only `Contended`.
#[derive(Debug)]
enum LeaseAttempt {
    Owned(File),
    /// A live process holds the lease.
    Contended,
    /// The attempt failed for a reason polling cannot cure.
    Failed(std::io::Error),
}

#[derive(Debug)]
pub(crate) enum NativeDesktopLeaseFailure {
    Deadline,
    Attempt(std::io::Error),
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

impl NativeDesktopLease {
    pub(super) fn acquire(deadline: Instant) -> Result<Self, NativeDesktopLeaseFailure> {
        let path = platform::lease_path();
        loop {
            match platform::attempt(&path) {
                LeaseAttempt::Owned(file) => {
                    return Ok(Self {
                        file: Some(file),
                        path,
                        owner_process_id: std::process::id(),
                    });
                }
                LeaseAttempt::Contended if Instant::now() < deadline => {
                    thread::sleep(ACQUISITION_POLL_INTERVAL);
                }
                LeaseAttempt::Contended => return Err(NativeDesktopLeaseFailure::Deadline),
                LeaseAttempt::Failed(error) => {
                    return Err(NativeDesktopLeaseFailure::Attempt(error));
                }
            }
        }
    }

    pub(super) fn owner_process_id(&self) -> u32 {
        self.owner_process_id
    }
}

impl Drop for NativeDesktopLease {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            platform::release(file, &self.path);
        }
    }
}

impl fmt::Display for NativeDesktopLeaseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline => formatter.write_str("deadline elapsed while another owner held it"),
            Self::Attempt(error) => write!(formatter, "attempt failed: {error}"),
        }
    }
}

#[cfg(all(test, worth_ui_product_executable))]
mod tests {
    use std::time::{Duration, Instant};

    use super::{NativeDesktopCourtroomLease, NativeDesktopLease, NativeDesktopLeaseFailure};

    #[test]
    fn exclusive_desktop_lease_rejects_a_concurrent_owner() {
        const CHILD_MARKER: &str = "WORTH_UI_DESKTOP_LEASE_CHILD";
        if std::env::var_os(CHILD_MARKER).is_some() {
            let failure = NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(20))
                .err()
                .expect("a live owner blocks the contender");
            assert!(
                matches!(failure, NativeDesktopLeaseFailure::Deadline),
                "contention must be the deadline, not an unrelated io failure: {failure}"
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
