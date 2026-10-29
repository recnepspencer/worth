//! The empty NoRelease selected checkpoint has no release controls, but its
//! tier-root custody still requires a genuine fresh C8-to-Serving handoff.

pub use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, crate::production_entry::certified_release_serving::open_serving_with_seal, crate::production_entry::certified_release_serving::recovery_request::request};

pub fn run_empty_no_release() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(
        "empty-no-release-serving",
    ).expect("fresh empty Store");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("root-only durable source for an empty Store checkpoint");
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x71; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(checkpoint).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(cause) => panic!("empty checkpoint denied: {cause:?}"),
        worth_proof::TransitionOutcome::Deferred(cause) => panic!("empty checkpoint deferred: {cause:?}"),
        worth_proof::TransitionOutcome::Stale(cause) => panic!("empty checkpoint stale: {cause:?}"),
        worth_proof::TransitionOutcome::RebindRequired(cause) => {
            panic!("empty checkpoint requires rebind: {cause:?}")
        }
        worth_proof::TransitionOutcome::Failed(cause) => panic!("empty checkpoint failed: {cause:?}"),
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let worker = std::thread::Builder::new()
        .name("empty-no-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::certification_recover_with_custody_pauses(
                request(&root),
                |_| {},
                || {},
            );
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine empty NoRelease custody failed: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("selected empty NoRelease seal");
            open_serving_with_seal(&root, seal);
        })
        .expect("empty recovery worker");
    worker.join().expect("empty recovery worker did not panic");
}