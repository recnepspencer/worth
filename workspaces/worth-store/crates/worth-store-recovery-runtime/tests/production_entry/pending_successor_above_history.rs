//! A pending V3 batch that names a per-object predecessor, killed at its
//! durable descriptor WAL. Serving admitted every earlier release without a
//! new checkpoint, so recovery must admit the pending batch from the same
//! checkpoint and ordered history, and then rejoin it as completed history.

use super::*;
use std::{fs, path::Path};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveredPhysicalRuntimeHandoff, WorthStoreRecovery,
};

fn recover(root: &Path, stage: &str) -> RecoveredPhysicalRuntimeHandoff {
    match WorthStoreRecovery::recover(certified_release_serving::request(root)) {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "{stage} blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
            block.kind,
            block.evidence().artifact.as_deref(),
            block.evidence().planning_denial,
            block.recovery_effects()
        ),
        PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
            "{stage} indeterminate: handoff={:?}; reopen={:?}; effects={}",
            failure.handoff_failure(),
            failure.reopen_failure(),
            failure.recovery_effects()
        ),
        _ => panic!("{stage} did not recover"),
    }
}

/// Recovers the parked batch, seals Serving on it, and publishes a
/// head-bearing checkpoint whose source roster carries every released object.
fn recover_and_checkpoint(root: &Path, stage: &str, key: [u8; 32]) -> usize {
    let seal = recover(root, stage).into_core().into_checkpoint_custody();
    let serving = certified_release_serving::admit_serving_with_seal(
        root,
        seal.expect("recovered batch custody seal"),
    );
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("{stage}: head-bearing checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let checkpoint = fs::read(root.join("families/checkpoint.current")).unwrap();
    let (_, accumulator) = release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    let heads = release_reopen::selected_head_oracle::selected_heads(root, accumulator);
    assert!(heads.iter().all(|head| !head.terminal()));
    heads.len()
}

/// The pending batch must redo and seal, and the next recovery must rejoin
/// it as completed history, both from the unchanged checkpoint.
fn assert_pending_then_completed(world: &pending_wal_world::PendingWalWorld, stage: &str) {
    let root = world.root();
    let checkpoint = fs::read(root.join("families/checkpoint.current")).unwrap();
    let pending = recover(root, stage);
    assert!(pending.core().recovery_effect_count() > 0);
    let seal = pending
        .into_core()
        .into_checkpoint_custody()
        .expect("Store must seal the pending successor");
    certified_release_serving::admit_serving_with_seal(root, seal).close();
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "{stage}: the redone successor must stay above the same checkpoint",
    );
    let completed = recover(root, stage);
    assert_eq!(completed.core().recovery_effect_count(), 0);
    let seal = completed
        .into_core()
        .into_checkpoint_custody()
        .expect("Store must seal the completed successor");
    certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}

/// The checkpoint heads the first object. Its second batch and a first
/// release of another object complete above the checkpoint; the first
/// object's third batch, whose predecessor is that ordered second batch, is
/// killed. Its chain leaves the ordered releases at the checkpoint head.
#[test]
fn pending_successor_of_an_ordered_release_above_a_head_checkpoint_recovers() {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    world.kill_distinct_release_before_checkpoint();
    world.kill_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor of an ordered release");
}

/// The checkpoint heads both objects. The first object's next batch completes
/// above it; the other object's next batch, whose predecessor is its own
/// checkpoint head, is killed above that foreign history.
#[test]
fn pending_successor_of_a_checkpoint_head_above_foreign_history_recovers() {
    let world = pending_wal_world::first();
    let root = world.root();
    assert_eq!(recover_and_checkpoint(root, "first object", [0xd1; 32]), 1);
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        recover_and_checkpoint(root, "distinct object", [0xd2; 32]),
        2
    );
    world.kill_successor_of_first_object();
    world.kill_successor_of_distinct_object();
    assert_pending_then_completed(&world, "pending successor of a checkpoint head");
}

/// The killed batch extends the checkpoint head directly and is terminal, so
/// its head effect replaces the head it replayed with a terminal entry.
#[test]
fn pending_terminal_successor_of_a_checkpoint_head_recovers() {
    let world = pending_wal_world::first();
    assert_eq!(
        recover_and_checkpoint(world.root(), "first object", [0xd3; 32]),
        1
    );
    world.kill_terminal_successor_of_first_object();
    assert_pending_then_completed(&world, "pending terminal successor of a checkpoint head");
}

/// No checkpoint heads the object: its first release completed above the
/// checkpoint, and the killed second batch names that ordered release.
#[test]
fn pending_successor_of_a_first_release_without_a_checkpoint_head_recovers() {
    let world = pending_wal_world::first();
    world.kill_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor of an ordered first release");
}
