pub(crate) use std::path::Path;

pub(crate) use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage;

pub(crate) use crate::process::support::production_harness::spawn_recovery_at_yieldpoint_with_deadline;
pub(crate) use worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard;

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
