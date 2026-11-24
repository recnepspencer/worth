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
    run_and_kill_observing_with_ctx(test, environment, announced, finished, || {}, test)
}

pub(crate) fn run_and_kill_with_ctx(
    test: &str,
    environment: &[(&str, &Path)],
    announced: &Path,
    finished: &Path,
    ctx: &str,
) -> ChildEnd {
    run_and_kill_observing_with_ctx(test, environment, announced, finished, || {}, ctx)
}

pub(crate) fn run_and_kill_observing(
    test: &str,
    environment: &[(&str, &Path)],
    announced: &Path,
    finished: &Path,
    on_park: impl FnOnce(),
) -> ChildEnd {
    run_and_kill_observing_with_ctx(test, environment, announced, finished, on_park, test)
}

pub(crate) fn run_and_kill_observing_with_ctx(
    test: &str,
    environment: &[(&str, &Path)],
    announced: &Path,
    finished: &Path,
    on_park: impl FnOnce(),
    ctx: &str,
) -> ChildEnd {
    let mut on_park = Some(on_park);
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--nocapture"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (name, value) in environment {
        command.env(name, value);
    }
    let mut child = command
        .spawn()
        .unwrap_or_else(|e| panic!("{ctx}: spawn {test} failed: {e:?}"));
    let deadline = std::time::Instant::now() + PARK_DEADLINE;
    loop {
        if announced.is_file() {
            on_park.take().unwrap()();
            let pre = child
                .try_wait()
                .unwrap_or_else(|e| panic!("{ctx}: poll {test} before kill: {e:?}"));
            assert!(
                pre.is_none(),
                "{ctx}: child {test} exited before parent OS kill: {pre:?}"
            );
            child
                .kill()
                .unwrap_or_else(|e| panic!("{ctx}: OS kill {test} failed: {e:?}"));
            let status = child
                .wait()
                .unwrap_or_else(|e| panic!("{ctx}: wait {test} after kill: {e:?}"));
            assert_os_killed(status, ctx);
            return crate::killed_child::ChildEnd::KilledAtPark;
        }
        if child
            .try_wait()
            .unwrap_or_else(|e| panic!("{ctx}: poll {test}: {e:?}"))
            .is_some()
        {
            let output = child
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{ctx}: wait_with_output on early-exit {test}: {e:?}"));
            assert!(
                finished.is_file() && !announced.is_file(),
                "{ctx}: child {test} ended without parking: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return crate::killed_child::ChildEnd::Finished;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{ctx}: wait_with_output on timed-out {test}: {e:?}"));
            panic!(
                "{ctx}: child {test} did not park: {}\n{}",
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

/// Verify that `status` reflects an OS-induced termination (`SIGKILL` on Unix,
/// `TerminateProcess` exit code `1` on Windows — rejecting normal `0` and Rust
/// panic exit code `101`).
pub(crate) fn assert_os_killed(status: std::process::ExitStatus, context: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(
            status.signal(),
            Some(9),
            "{context}: child must end via SIGKILL (9), got {status:?}"
        );
    }
    #[cfg(windows)]
    {
        assert_eq!(
            status.code(),
            Some(1),
            "{context}: child must end via OS TerminateProcess (exit code 1, not 0 or panic 101), got {status:?}"
        );
    }
    #[cfg(not(any(unix, windows)))]
    {
        assert!(
            !status.success(),
            "{context}: killed child must not exit cleanly: {status:?}"
        );
    }
}

/// Park the calling child forever; only the parent's kill ends it.
pub(crate) fn park() -> ! {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
