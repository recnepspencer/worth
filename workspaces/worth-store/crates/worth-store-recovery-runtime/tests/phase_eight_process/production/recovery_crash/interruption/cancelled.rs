use std::path::Path;

use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage;

use super::super::super::harness::spawn_recovery_at_yieldpoint;
use crate::child_lifecycle::ProcessChildGuard;

pub(super) fn spawn(
    root: &Path,
    report: &Path,
    temporary_root: &Path,
    stage: PhysicalRecoveryYieldpointStage,
    reached: &Path,
    release: &Path,
) -> ProcessChildGuard {
    spawn_recovery_at_yieldpoint(root, report, temporary_root, stage, reached, release)
}
