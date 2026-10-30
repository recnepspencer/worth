//! Production recovery and Store construction, with no certification pause.
//! The feature is used only to create authoritative release/tier fixtures.

pub(crate) use worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution;
pub(crate) use {
    worth_proof::TransitionOutcome, worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) fn checkpoint(world: &PhysicalResidencyStoreWorld, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("production-custody fixture checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub(crate) fn recover_and_open(world: PhysicalResidencyStoreWorld) -> u64 {
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let worker = std::thread::Builder::new()
        .name("production-custody-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome =
                worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root));
            if let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = &outcome {
                panic!(
                    "production C8→Store custody handoff denied: {:?}",
                    indeterminate.handoff_failure()
                );
            }
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("production C8→Store custody denied: {outcome:?}")
            };
            let recovered_generation = handoff.core().root().generation();
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("production selected custody seal");
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal(&root, seal);
            recovered_generation
        })
        .expect("production recovery worker");
    worker.join().expect("production recovery worker")
}
