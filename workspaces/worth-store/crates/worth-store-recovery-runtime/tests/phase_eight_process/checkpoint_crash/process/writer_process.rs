use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::super::super::child_lifecycle::ProcessChildGuard;
use super::super::super::history::{c8_writer_binary_path, SubmittedOperationProgram};
use super::super::CheckpointCrashStage;

pub(crate) fn spawn_writer(
    stage: CheckpointCrashStage,
    schedule_seed: u64,
    perturbation_seed: u64,
    root: &Path,
    operation_program: &SubmittedOperationProgram,
    start: &Path,
    reached: &Path,
) -> ProcessChildGuard {
    ProcessChildGuard::new(
        Command::new(c8_writer_binary_path())
            .args(["--root"])
            .arg(root)
            .args(["--start-marker"])
            .arg(start)
            .args(["--reached-marker"])
            .arg(reached)
            .args(["--checkpoint-stage"])
            .arg(format!(
                "{}@{schedule_seed}:{perturbation_seed}",
                stage.label()
            ))
            .args(["--writer-durability-profile"])
            .arg(operation_program.writer_profile_selection.cli_name())
            .args(["--operation-program"])
            .arg(&operation_program.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn killed checkpoint writer"),
    )
}

pub(crate) fn wait_for_writer_ready(child: &mut ProcessChildGuard, start_marker: &Path) {
    let ready_marker = start_marker.with_extension("ready");
    let deadline = Instant::now() + Duration::from_secs(120);
    while !ready_marker.exists() {
        assert!(Instant::now() < deadline, "checkpoint root timeout");
        if let Some(status) = child
            .child_mut()
            .try_wait()
            .expect("poll checkpoint writer")
        {
            let (stdout, stderr) = exited_output(child);
            panic!(
                "checkpoint writer exited before root initialization: status={status}, stdout={}, stderr={}",
                stdout,
                stderr
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(crate) fn wait_for_marker(child: &mut ProcessChildGuard, marker: &Path, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !marker.exists() {
        assert!(Instant::now() < deadline, "{label} marker timeout");
        if let Some(status) = child.child_mut().try_wait().expect("poll checkpoint child") {
            let (stdout, stderr) = exited_output(child);
            panic!(
                "checkpoint child exited before {label}: status={status}, stdout={}, stderr={}",
                stdout, stderr
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn exited_output(child: &mut ProcessChildGuard) -> (String, String) {
    let process = child.child_mut();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(stream) = process.stdout.as_mut() {
        let _ = stream.read_to_end(&mut stdout);
    }
    if let Some(stream) = process.stderr.as_mut() {
        let _ = stream.read_to_end(&mut stderr);
    }
    (
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}
