//! Production recovery and Store construction, with no certification pause.
//! The feature is used only to create authoritative release/tier fixtures.

use super::*;
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};
use worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution;

fn checkpoint(world: &PhysicalResidencyStoreWorld, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("production-custody fixture checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn recover_and_open(world: PhysicalResidencyStoreWorld) -> u64 {
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let worker = std::thread::Builder::new()
        .name("production-custody-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome =
                WorthStoreRecovery::recover(super::certified_release_serving::request(&root));
            if let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = &outcome {
                panic!(
                    "production C8→Store custody handoff denied: {:?}",
                    indeterminate.handoff_failure()
                );
            }
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("production C8→Store custody denied: {outcome:?}")
            };
            let recovered_generation = handoff.core().root().generation();
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("production selected custody seal");
            super::certified_release_serving::open_serving_with_seal(&root, seal);
            recovered_generation
        })
        .expect("production recovery worker");
    worker.join().expect("production recovery worker")
}

#[test]
fn production_release_custody_opens_serving_after_fresh_recovery() {
    let (world, _, _) = release_reopen::released_world(1024);
    checkpoint(&world, 0xc1);
    recover_and_open(world);
}

#[test]
fn production_two_distinct_releases_open_serving_after_carryforward() {
    let (world, _) = release_reopen::two_generations::world();
    checkpoint(&world, 0xc2);
    recover_and_open(world);
}

#[test]
fn production_mixed_failed_ingest_and_release_open_serving() {
    let (world, _) = release_reopen::mixed_failed_ingest::world();
    checkpoint(&world, 0xc3);
    recover_and_open(world);
}

#[test]
fn production_no_release_custody_opens_serving() {
    let world = initialized_recovery_world("production-no-release-custody");
    checkpoint(&world, 0xc4);
    recover_and_open(world);
}

#[test]
fn production_tier_no_release_custody_opens_serving() {
    let world = initialized_recovery_world("production-tier-no-release-custody");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("one-time tier epoch");
    checkpoint(&world, 0xc5);
    recover_and_open(world);
}

#[test]
fn production_tier_no_release_rebinds_after_selected_wal_redo() {
    let world = initialized_recovery_world("production-tier-no-release-wal-redo");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("one-time tier activation");
    checkpoint(&world, 0xc7);
    let selected_generation = world
        .serving()
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get();
    canonical_durable_wal_attempt_without_execution(
        &world,
        [0xd7; 32],
        b"post-checkpoint-tier-selected-redo",
    );
    assert!(
        recover_and_open(world) > selected_generation,
        "selected WAL redo must advance the root bound to the serving seal"
    );
}

#[test]
fn production_tier_and_release_custody_open_serving() {
    let (world, _, _) = release_reopen::released_tier_world(1024);
    checkpoint(&world, 0xc6);
    recover_and_open(world);
}
