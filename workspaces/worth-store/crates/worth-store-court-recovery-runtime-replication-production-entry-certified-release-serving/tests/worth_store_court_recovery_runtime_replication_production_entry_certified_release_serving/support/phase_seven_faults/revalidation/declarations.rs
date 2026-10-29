pub(crate) use {worth_store::physical_runtime::certification::MediaFaultDirective};
pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryCleanupRemovalDenialKind, worth_store::physical_runtime::RecoveryCleanupArtifactRevalidationDenial, worth_store::physical_runtime::RecoveryCleanupRemovalDenialCause};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence, worth_store_recovery_runtime::RecoveryCleanupPosture};

pub(crate) use {crate::phase_seven_faults::world::cleanup_fault, crate::phase_seven_faults::world::cleanup_world, crate::phase_seven_faults::world::empty_fault_schedule, crate::phase_seven_faults::world::recover_with_schedule, crate::phase_seven_faults::world::reopen_with_schedule};

pub(crate) fn current_checkpoint_bytes(root: &std::path::Path) -> u64 {
    std::fs::metadata(root.join("families").join("checkpoint.current"))
        .unwrap()
        .len()
}