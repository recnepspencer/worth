//! A public diagnostic observer shares the real recovered handoff's storage.

pub(super) fn share_without_allocation(
    handoff: &worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
) -> worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservations {
    let allocations = handoff.core().certification_residency_allocations();
    let before = allocations.snapshot();
    let observations = handoff.wal_integrity_observation_storage().clone();
    assert_eq!(
        observations.wal().as_ptr(),
        handoff.wal_integrity_observations().as_ptr(),
        "a concurrent diagnostic observer shares the immutable storage"
    );
    assert_eq!(
        allocations.snapshot(),
        before,
        "cloning needs no new allocation"
    );
    observations
}
