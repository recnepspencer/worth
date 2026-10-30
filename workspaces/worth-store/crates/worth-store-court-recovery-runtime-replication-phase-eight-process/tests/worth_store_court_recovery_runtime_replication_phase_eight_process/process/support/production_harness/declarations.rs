pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Output};

pub(crate) fn run_observer(root: &Path, output: PathBuf, temporary_root: &Path) -> (u32, Output) {
    let mut command = std::process::Command::new(
        crate::support_binaries::phase_eight_process_binaries()
            .observer()
            .path(),
    );
    command
        .arg("c8-recovery-observe")
        .arg(root)
        .arg(output)
        .args(["32768", "16384", "16384", "536870912"])
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    run_process(command)
}

pub(crate) fn run_recovery_with_profile(
    root: &Path,
    report: &Path,
    temporary_root: &Path,
    profile: &str,
) -> (u32, Output) {
    let mut command = std::process::Command::new(
        crate::support_binaries::phase_eight_process_binaries()
            .recovery()
            .path(),
    );
    command
        .arg(root)
        .arg(format!("--bounded-profile={profile}"))
        .arg(format!("--report={}", report.display()))
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    run_process(command)
}

pub(crate) fn spawn_recovery_at_yieldpoint(
    root: &Path,
    report: &Path,
    temporary_root: &Path,
    stage: worth_store::physical_runtime::PhysicalRecoveryYieldpointStage,
    reached: &Path,
    release: &Path,
) -> worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard {
    spawn_recovery_at_yieldpoint_with_deadline(
        root,
        report,
        temporary_root,
        stage,
        reached,
        release,
        None,
    )
}

pub(crate) fn spawn_recovery_at_yieldpoint_with_deadline(
    root: &Path,
    report: &Path,
    temporary_root: &Path,
    stage: worth_store::physical_runtime::PhysicalRecoveryYieldpointStage,
    reached: &Path,
    release: &Path,
    deadline_ms: Option<u64>,
) -> worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard {
    let mut command = std::process::Command::new(
        crate::support_binaries::phase_eight_process_binaries()
            .recovery()
            .path(),
    );
    command
        .arg(root)
        .arg("--bounded-profile=c8-phase8-fate-coverage-v1")
        .arg(format!("--report={}", report.display()))
        .arg(format!("--yieldpoint-stage={}", stage.label()))
        .arg(format!("--yieldpoint-reached={}", reached.display()))
        .arg(format!("--yieldpoint-release={}", release.display()))
        .arg(format!(
            "--yieldpoint-cancel={}",
            release.with_extension("cancel").display()
        ));
    if let Some(deadline_ms) = deadline_ms {
        command.arg(format!("--yieldpoint-deadline-ms={deadline_ms}"));
    }
    let child = command
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root)
        .spawn()
        .expect("launch recovery process yieldpoint child");
    worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard::new(child)
}

pub(crate) fn run_process(mut command: Command) -> (u32, Output) {
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command
        .spawn()
        .map(worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard::new)
        .expect("launch Phase 8 production process");
    let process_id = child.id();
    let output = child
        .wait_with_output_within(std::time::Duration::from_secs(120))
        .expect("wait for Phase 8 production process");
    (process_id, output)
}

pub(crate) fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
