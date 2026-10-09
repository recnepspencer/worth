//! A pending V3 batch that names a per-object predecessor, killed at its
//! durable descriptor WAL. Serving admitted every earlier release without a
//! new checkpoint, so recovery must admit the pending batch from the same
//! checkpoint and ordered history, and then rejoin it as completed history.

use super::*;
use pending_wal_world::PendingWalWorld;
use std::fs;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome, RecoveredPhysicalRuntimeHandoff,
    WorthStoreRecovery,
};

/// What the first reopen of the frontier world needs. Fixed: about 140 root
/// steps stand above its checkpoint, and each is charged what its member
/// declares it wrote, whichever blocks the ingest landed in.
/// Every root read costs one entry, found or not. Among them: the successor
/// root the candidate probe finds absent (1), and the historical roots a
/// blob record is looked up under (2).
const FRONTIER_NEED: u64 = 1003;

pub(super) fn recover(world: &PendingWalWorld, stage: &str) -> RecoveredPhysicalRuntimeHandoff {
    match WorthStoreRecovery::recover(world.recovery_request()) {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "{stage} blocked: kind={:?}; artifact={:?}; limit={:?}; sources={:?}; cause={:?}; \
             effects={}",
            block.cause(),
            block.evidence().artifact.as_deref(),
            block.cause().limit(),
            block.evidence().source_denials,
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

/// One manifest entry short of `need` is that limit, reached by the refused
/// charge, and `need` recovers. Neither attempt is sealed, so the world is
/// left for the recovery that is.
fn assert_needs_exactly(world: &PendingWalWorld, need: u64, stage: &str) {
    let recover = |entries| {
        WorthStoreRecovery::recover(
            certified_release_serving::request_for_long_ingest_with_manifest_entries(
                world.root(),
                entries,
            ),
        )
    };
    let PhysicalRecoveryOutcome::Blocked(blocked) = recover(need - 1) else {
        panic!("{stage}: {} entries must block", need - 1)
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.cause().limit().map(|limit| (
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        )),
        Some((
            PhysicalRecoveryLimitDimension::ManifestEntries,
            need,
            need - 1
        )),
        "{stage}: denial={:?}",
        blocked.evidence().planning_denial,
    );
    let PhysicalRecoveryOutcome::Recovered(handoff) = recover(need) else {
        panic!("{stage}: {need} entries must recover")
    };
    drop(handoff);
}

/// Recovers the parked batch, seals Serving on it, and publishes a
/// head-bearing checkpoint whose source roster carries every released object.
fn recover_and_checkpoint(world: &PendingWalWorld, stage: &str, key: [u8; 32]) -> usize {
    let heads = checkpoint_heads(world, stage, key);
    assert!(heads.iter().all(|head| !head.terminal()));
    heads.len()
}

/// The heads of the checkpoint published over the recovered world.
fn checkpoint_heads(
    world: &PendingWalWorld,
    stage: &str,
    key: [u8; 32],
) -> Vec<worth_store_physical_format::ReleaseCustodyHeadEntryV1> {
    let root = world.root();
    let seal = recover(world, stage).into_core().into_checkpoint_custody();
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
    release_reopen::selected_head_oracle::selected_heads(root, accumulator)
}

/// The pending batch must redo and seal, and the next recovery must rejoin
/// it as completed history, both from the unchanged checkpoint.
fn assert_pending_then_completed(world: &PendingWalWorld, stage: &str) {
    let root = world.root();
    let checkpoint = fs::read(root.join("families/checkpoint.current")).unwrap();
    let pending = recover(world, stage);
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
    let completed = recover(world, stage);
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
    let world = ordered_release_above_a_head_checkpoint();
    assert_pending_then_completed(&world, "pending successor of an ordered release");
}

pub(super) fn ordered_release_above_a_head_checkpoint() -> PendingWalWorld {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    world.kill_distinct_release_before_checkpoint();
    world.kill_successor_of_first_object();
    world
}

/// The checkpoint heads both objects. The first object's next batch completes
/// above it; the other object's next batch, whose predecessor is its own
/// checkpoint head, is killed above that foreign history.
#[test]
fn pending_successor_of_a_checkpoint_head_above_foreign_history_recovers() {
    let world = successor_of_a_checkpoint_head_above_foreign_history();
    assert_pending_then_completed(&world, "pending successor of a checkpoint head");
}

pub(super) fn successor_of_a_checkpoint_head_above_foreign_history() -> PendingWalWorld {
    let world = pending_wal_world::first();
    assert_eq!(
        recover_and_checkpoint(&world, "first object", [0xd1; 32]),
        1
    );
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        recover_and_checkpoint(&world, "distinct object", [0xd2; 32]),
        2
    );
    world.kill_successor_of_first_object();
    world.kill_successor_of_distinct_object();
    world
}

/// The killed batch extends the checkpoint head directly and is terminal, so
/// its head effect replaces the head it replayed with a terminal entry.
#[test]
fn pending_terminal_successor_of_a_checkpoint_head_recovers() {
    let world = terminal_successor_of_a_checkpoint_head();
    assert_pending_then_completed(&world, "pending terminal successor of a checkpoint head");
}

/// A checkpoint heads the first object, and the batch that would complete
/// its release is killed above that checkpoint.
pub(super) fn terminal_successor_of_a_checkpoint_head() -> PendingWalWorld {
    let world = pending_wal_world::first();
    assert_eq!(
        recover_and_checkpoint(&world, "first object", [0xd3; 32]),
        1
    );
    world.kill_full_batch_successor_of_first_object();
    world
}

/// No checkpoint heads the object: its first release completed above the
/// checkpoint, and the killed second batch names that ordered release.
#[test]
fn pending_successor_of_a_first_release_without_a_checkpoint_head_recovers() {
    let world = pending_wal_world::first();
    world.kill_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor of an ordered first release");
}

/// The object checkpointed two resume frontiers during its ingest. The first
/// batch drops only its publication and is headed by a checkpoint. The next
/// batch, killed, drops both frontiers and the chunks beside them; the last
/// one, killed, drops the chunks left and their root node, so the head
/// recovered from it is terminal.
#[test]
fn pending_successors_of_a_generation_with_a_resume_frontier_recover_to_a_terminal_head() {
    let world = pending_wal_world::first_with_resume_frontier();
    assert_needs_exactly(&world, FRONTIER_NEED, "frontier object");
    assert_eq!(
        recover_and_checkpoint(&world, "frontier object", [0xd4; 32]),
        1
    );
    world.kill_full_batch_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor that drops a resume frontier");
    world.kill_full_batch_successor_of_first_object();
    assert_pending_then_completed(&world, "pending terminal successor after a resume frontier");
    let heads = checkpoint_heads(&world, "released frontier object", [0xd5; 32]);
    assert_eq!(heads.len(), 1);
    assert!(
        heads[0].terminal(),
        "the release stayed nonterminal: {heads:?}"
    );
}
