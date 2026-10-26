use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[path = "phase_eight_process_suite/child.rs"]
mod child;

/// Runs the Phase 8 fresh-process suite. The suite builds its writer,
/// observer, and recovery executables itself, each independently of the test
/// image, so the runner only bounds the run.
pub(super) fn run(workspace: &Path, target_root: Option<&Path>) -> Result<(), String> {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(workspace).args([
        "test",
        "--locked",
        "-p",
        "worth-store-recovery-runtime",
        "--test",
        "phase_eight_process",
        "--features",
        "certification-test-authority",
    ]);
    command.env("CARGO_TARGET_DIR", cargo_target(workspace, target_root));
    successful(
        child::run_within(&mut command, Duration::from_secs(60 * 60))?,
        "Phase 8 process suite",
    )
}

fn cargo_target(workspace: &Path, target_root: Option<&Path>) -> PathBuf {
    target_root
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from))
        .map(|target| {
            if target.is_absolute() {
                target
            } else {
                workspace.join(target)
            }
        })
        .unwrap_or_else(|| workspace.join("target"))
}

fn successful(status: std::process::ExitStatus, action: &str) -> Result<(), String> {
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{action} exited with {status}"))
}
