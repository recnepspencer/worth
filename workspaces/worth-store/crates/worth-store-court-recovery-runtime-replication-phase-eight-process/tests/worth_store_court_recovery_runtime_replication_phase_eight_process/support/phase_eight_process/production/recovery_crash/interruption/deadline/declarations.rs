pub(crate) use {std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryYieldpointStage};

pub(crate) use {crate::phase_eight_process::production::harness::process::spawn_recovery_at_yieldpoint_with_deadline};
pub(crate) use {worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard};

pub(crate) fn spawn(
    root: &Path,
    report: &Path,
    temporary_root: &Path,
    stage: PhysicalRecoveryYieldpointStage,
    reached: &Path,
    release: &Path,
) -> ProcessChildGuard {
    spawn_recovery_at_yieldpoint_with_deadline(
        root,
        report,
        temporary_root,
        stage,
        reached,
        release,
        Some(0),
    )
}