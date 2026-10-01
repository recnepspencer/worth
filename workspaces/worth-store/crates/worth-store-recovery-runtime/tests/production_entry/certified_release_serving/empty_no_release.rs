//! The empty NoRelease selected checkpoint has no release controls, but its
//! tier-root custody still requires a genuine fresh C8-to-Serving handoff.

use super::*;

pub(crate) fn run_empty_no_release() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(
        "empty-no-release-serving",
    ).expect("fresh empty Store");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("root-only durable source for an empty Store checkpoint");
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x71; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(checkpoint).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Denied(cause) => panic!("empty checkpoint denied: {cause:?}"),
        TransitionOutcome::Deferred(cause) => panic!("empty checkpoint deferred: {cause:?}"),
        TransitionOutcome::Stale(cause) => panic!("empty checkpoint stale: {cause:?}"),
        TransitionOutcome::RebindRequired(cause) => {
            panic!("empty checkpoint requires rebind: {cause:?}")
        }
        TransitionOutcome::Failed(cause) => panic!("empty checkpoint failed: {cause:?}"),
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let worker = std::thread::Builder::new()
        .name("empty-no-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
                request(&root),
                |_| {},
                || {},
            );
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine empty NoRelease custody failed: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("selected empty NoRelease seal");
            open_serving_with_seal(&root, seal);
        })
        .expect("empty recovery worker");
    worker.join().expect("empty recovery worker did not panic");
}
