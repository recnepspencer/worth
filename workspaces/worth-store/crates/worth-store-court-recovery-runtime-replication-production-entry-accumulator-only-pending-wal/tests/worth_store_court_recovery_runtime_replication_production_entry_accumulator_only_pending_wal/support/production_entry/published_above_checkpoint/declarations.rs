//! Objects published above the selected checkpoint by a process killed
//! before the next one. Their index maintenance filled whole inline pages
//! and retired every record on them, so recovery meets WAL images of pages
//! the selected root no longer routes.

pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::published_above_checkpoint};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld};
pub(crate) use {std::fs};





/// Recovers in this fresh process and reads every object back, does so again
/// above the same checkpoint, then checkpoints and requires the next recovery
/// to find nothing left to do.
pub(crate) fn assert_recovers_reads_back_and_reopens(workload: Workload, tail: Tail, key: u8) {
    let world = pending_wal_world::published_above_checkpoint(workload, tail);
    let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let serving = serve(&world, "first recovery");
    world.assert_objects_read_back(&serving);
    serving.close();
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "the recovered publications must stay above the same checkpoint",
    );
    let serving = serve(&world, "recovery above the same checkpoint");
    world.assert_objects_read_back(&serving);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("checkpoint over the recovered publications must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let reopened = worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::recover(&world, "second recovery");
    assert_eq!(reopened.core().recovery_effect_count(), 0);
    let seal = reopened
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("checkpointed publications custody seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    world.assert_objects_read_back(&serving);
    serving.close();
}

pub(crate) fn serve(
    world: &PendingWalWorld,
    stage: &str,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let seal = worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::recover(world, stage)
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("recovered publications custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal)
}