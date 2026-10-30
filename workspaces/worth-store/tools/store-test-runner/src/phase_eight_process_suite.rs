use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[path = "phase_eight_process_suite/child.rs"]
mod child;

/// Runs the moved Phase 8 courts through the shared executable prebuilder.
pub(super) fn run(workspace: &Path, target_root: Option<&Path>) -> Result<(), String> {
    let repository = workspace
        .parent()
        .and_then(Path::parent)
        .ok_or("Store workspace must live under workspaces/")?;
    let mut command =
        Command::new(std::env::var_os("WORTH_PYTHON").unwrap_or_else(|| "python".into()));
    command.current_dir(repository)
        .arg(repository.join("scripts/test_cost/run_lane.py"))
        .args(["--workspace", "store", "--filter",
               "binary_id(=worth-store-court-recovery-runtime-replication-phase-eight-process::worth_store_court_recovery_runtime_replication_phase_eight_process)"]);
    if let Some(target) = target_root {
        command.env("CARGO_TARGET_DIR", target);
    }
    successful(
        child::run_within(&mut command, Duration::from_secs(60 * 60))?,
        "Phase 8 process suite",
    )
}

fn successful(status: std::process::ExitStatus, action: &str) -> Result<(), String> {
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{action} exited with {status}"))
}
