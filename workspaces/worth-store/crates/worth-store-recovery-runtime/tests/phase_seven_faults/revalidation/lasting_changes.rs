//! A checkpoint change made before `finish` stops at Store's C.11 residue
//! gate (C.11 plan, Phase 6 checkpoint release custody: Store independently
//! rereads the selected checkpoint before cleanup and before its Serving
//! seal), so the WAL cleanup candidate is never reached. WAL changes made
//! before `finish` do reach cleanup; `denial_matrix.rs` and
//! `cleanup_revalidates_exact_wal_bytes_before_deletion` pin them.

use worth_store::physical_runtime::{ArtifactTreeFailureKind, RecoveryCheckpointResidueDenial};
use worth_store_recovery_runtime::{PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome};

use super::super::world::{cleanup_world, empty_fault_schedule, reopen_with_schedule};
use super::checkpoint_replacement::replace_with_distinct_valid_checkpoint;
use super::window::{append_byte, checkpoint_path, flip_first_byte};

#[test]
fn missing_checkpoint_blocks_at_the_store_residue_gate() {
    let denial = residue_gate_denial("lasting-checkpoint-missing", |root| {
        std::fs::remove_file(checkpoint_path(root)).unwrap();
    });
    assert!(matches!(
        denial,
        RecoveryCheckpointResidueDenial::Observation(failure)
            if failure.kind() == ArtifactTreeFailureKind::Absent
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
        RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
    );
}

#[test]
fn malformed_checkpoint_blocks_at_the_store_residue_gate() {
    let denial = residue_gate_denial("lasting-checkpoint-malformed", |root| {
        flip_first_byte(&checkpoint_path(root));
    });
    assert_eq!(
        denial,
        RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
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
        RecoveryCheckpointResidueDenial::SelectedCheckpointMismatch
    );
    assert!(candidate.exists());
}

fn residue_gate_denial(
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

fn blocked_residue_denial(outcome: PhysicalRecoveryOutcome) -> RecoveryCheckpointResidueDenial {
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("a lasting checkpoint change must block at Store's residue gate")
    };
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::Checkpoint);
    assert_eq!(blocked.recovery_effects(), 0);
    blocked
        .evidence()
        .checkpoint_residue_denial
        .expect("the residue gate names its denial")
}
