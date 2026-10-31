//! A child process of this test binary, run as one `--exact` test with its
//! role in the environment, killed once it announces that it has parked.
//! The child announces by renaming a fully written file into place, so the
//! parent never reads a partial announcement.

pub(crate) use std::fs;
pub(crate) use std::path::Path;
pub(crate) use std::thread;
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) const PARK_DEADLINE: Duration = std::time::Duration::from_secs(240);

/// How a child ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ChildEnd {
    /// It parked and was killed there.
    KilledAtPark,
    /// It finished without parking.
    Finished,
}

/// Run `test` as a child with `environment`; kill it as soon as `announced`
/// exists. A child that exits without announcing must have written
/// `finished`, or the parent fails with its output.
pub(crate) fn run_and_kill(
    test: &str,
    environment: &[(&str, &Path)],
    announced: &Path,
    finished: &Path,
) -> ChildEnd {
    run_and_kill_observing(test, environment, announced, finished, || {})
}

pub(crate) fn run_and_kill_observing(
    test: &str,
    environment: &[(&str, &Path)],
    announced: &Path,
    finished: &Path,
    on_park: impl FnOnce(),
) -> ChildEnd {
    let mut on_park = Some(on_park);
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (name, value) in environment {
        command.env(name, value);
    }
    let mut child = command.spawn().unwrap();
    let deadline = std::time::Instant::now() + PARK_DEADLINE;
    loop {
        if announced.is_file() {
            on_park.take().unwrap()();
            child.kill().unwrap();
            child.wait().unwrap();
            return crate::killed_child::ChildEnd::KilledAtPark;
        }
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            assert!(
                finished.is_file() && !announced.is_file(),
                "child {test} ended without parking: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return crate::killed_child::ChildEnd::Finished;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "child {test} did not park: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Publish `bytes` at `path` in one rename.
pub(crate) fn announce(path: &Path, bytes: &[u8]) {
    let pending = path.with_extension("pending");
    std::fs::write(&pending, bytes).unwrap();
    std::fs::rename(pending, path).unwrap();
}

/// Park the calling child forever; only the parent's kill ends it.
pub(crate) fn park() -> ! {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
