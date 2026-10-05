//! Objects published above the selected checkpoint by a process killed
//! before the next one. Their index maintenance filled whole inline pages
//! and retired every record on them, so recovery meets WAL images of pages
//! the selected root no longer routes.

use super::*;
use pending_wal_world::{PendingWalWorld, Tail, Workload};
use std::fs;

#[path = "published_above_checkpoint/damaged_history.rs"]
mod damaged_history;
#[path = "published_above_checkpoint/manifest_entry_limit.rs"]
mod manifest_entry_limit;

/// Recovers in this fresh process and reads every object back, does so again
/// above the same checkpoint, then checkpoints and requires the next recovery
/// to find nothing left to do.
fn assert_recovers_reads_back_and_reopens(workload: Workload, tail: Tail, key: u8) {
    let world = pending_wal_world::published_above_checkpoint(workload, tail);
    let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let serving = serve(&world, "first recovery");
    world.assert_objects_read_back(&serving);
    serving.close();
    assert_eq!(
        fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "the recovered publications must stay above the same checkpoint",
    );
    let serving = serve(&world, "recovery above the same checkpoint");
    world.assert_objects_read_back(&serving);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("checkpoint over the recovered publications must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let reopened = pending_successor_above_history::recover(&world, "second recovery");
    assert_eq!(reopened.core().recovery_effect_count(), 0);
    let seal = reopened
        .into_core()
        .into_checkpoint_custody()
        .expect("checkpointed publications custody seal");
    let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
    world.assert_objects_read_back(&serving);
    serving.close();
}

fn serve(
    world: &PendingWalWorld,
    stage: &str,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let seal = pending_successor_above_history::recover(world, stage)
        .into_core()
        .into_checkpoint_custody()
        .expect("recovered publications custody seal");
    certified_release_serving::admit_serving_with_seal(world.root(), seal)
}

#[test]
fn one_24_chunk_object_published_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::TwentyFourChunks, Tail::Idle, 0xe1);
}

#[test]
fn three_2_chunk_objects_published_above_the_checkpoint_recover() {
    assert_recovers_reads_back_and_reopens(Workload::ThreeSmallObjects, Tail::Idle, 0xe2);
}

#[test]
fn one_66_chunk_object_published_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::SixtySixChunks, Tail::Idle, 0xe3);
}

#[test]
fn one_24_chunk_object_under_a_pending_release_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::TwentyFourChunks, Tail::PendingRelease, 0xe4);
}

#[test]
fn three_2_chunk_objects_under_a_pending_release_above_the_checkpoint_recover() {
    assert_recovers_reads_back_and_reopens(Workload::ThreeSmallObjects, Tail::PendingRelease, 0xe5);
}

#[test]
fn one_66_chunk_object_under_a_pending_release_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::SixtySixChunks, Tail::PendingRelease, 0xe6);
}
