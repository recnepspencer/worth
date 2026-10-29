//! A checkpoint change made before `finish` stops at Store's C.11 residue
//! gate (C.11 plan, Phase 6 checkpoint release custody: Store independently
//! rereads the selected checkpoint before cleanup and before its Serving
//! seal), so the WAL cleanup candidate is never reached. WAL changes made
//! before `finish` do reach cleanup; `denial_matrix.rs` and
//! `cleanup_revalidates_exact_wal_bytes_before_deletion` pin them.

#[allow(unused_imports)] use crate::phase_seven_faults::revalidation::lasting_changes::*;
use {worth_store::physical_runtime::ArtifactTreeFailureKind, worth_store::physical_runtime::RecoveryCheckpointResidueDenial};
use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOutcome};

use {crate::phase_seven_faults::world::cleanup_world, crate::phase_seven_faults::world::empty_fault_schedule, crate::phase_seven_faults::world::reopen_with_schedule};
use {crate::revalidation::checkpoint_replacement::replace_with_distinct_valid_checkpoint};
use {crate::revalidation::window::append_byte, crate::revalidation::window::checkpoint_path, crate::revalidation::window::flip_first_byte};

#[test]
fn missing_checkpoint_blocks_at_the_store_residue_gate() {
    let denial = residue_gate_denial("lasting-checkpoint-missing", |root| {
        std::fs::remove_file(checkpoint_path(root)).unwrap();
    });
    assert!(matches!(
        denial,
        worth_store::physical_runtime::RecoveryCheckpointResidueDenial::Observation(failure)
            if failure.kind() == worth_store::physical_runtime::ArtifactTreeFailureKind::Absent
                && failure.io_kind() == Some(std::io::ErrorKind::NotFound)
    ));
}

#[test]
fn oversized_checkpoint_blocks_at_the_store_residue_gate() {
    let denial = residue_gate_denial("lasting-checkpoint-oversized", |root| {
        append_byte(&checkpoint_path(root));
    });
    assert_eq!(
        denial,
        worth_store::physical_runtime::RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
    );
}

#[test]
fn malformed_checkpoint_blocks_at_the_store_residue_gate() {
    let denial = residue_gate_denial("lasting-checkpoint-malformed", |root| {
        flip_first_byte(&checkpoint_path(root));
    });
    assert_eq!(
        denial,
        worth_store::physical_runtime::RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
    );
}

#[test]
fn distinct_valid_checkpoint_blocks_at_the_store_residue_gate() {
    let world = cleanup_world("lasting-checkpoint-substitution");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    replace_with_distinct_valid_checkpoint(&world.root, reopened.store_identity());
    let denial = blocked_residue_denial(reopened.finish());
    assert_eq!(
        denial,
        worth_store::physical_runtime::RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
    );
    assert!(candidate.exists());
}
