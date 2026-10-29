//! A checkpoint change made before `finish` stops at Store's C.11 residue
//! gate (C.11 plan, Phase 6 checkpoint release custody: Store independently
//! rereads the selected checkpoint before cleanup and before its Serving
//! seal), so the WAL cleanup candidate is never reached. WAL changes made
//! before `finish` do reach cleanup; `denial_matrix.rs` and
//! `cleanup_revalidates_exact_wal_bytes_before_deletion` pin them.

pub(crate) use {worth_store::physical_runtime::ArtifactTreeFailureKind, worth_store::physical_runtime::RecoveryCheckpointResidueDenial};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOutcome};

pub(crate) use {crate::phase_seven_faults::world::cleanup_world, crate::phase_seven_faults::world::empty_fault_schedule, crate::phase_seven_faults::world::reopen_with_schedule};
pub(crate) use {crate::revalidation::checkpoint_replacement::replace_with_distinct_valid_checkpoint};
pub(crate) use {crate::revalidation::window::append_byte, crate::revalidation::window::checkpoint_path, crate::revalidation::window::flip_first_byte};

pub(crate) fn residue_gate_denial(
    label: &str,
    change: impl FnOnce(&std::path::Path),
) -> RecoveryCheckpointResidueDenial {
    let world = cleanup_world(label);
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    change(&world.root);
    let denial = blocked_residue_denial(reopened.finish());
    assert!(candidate.exists());
    denial
}

pub(crate) fn blocked_residue_denial(outcome: PhysicalRecoveryOutcome) -> RecoveryCheckpointResidueDenial {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("a lasting checkpoint change must block at Store's residue gate")
    };
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Checkpoint)
    );
    assert_eq!(blocked.recovery_effects(), 0);
    blocked
        .evidence()
        .checkpoint_residue_denial
        .expect("the residue gate names its denial")
}