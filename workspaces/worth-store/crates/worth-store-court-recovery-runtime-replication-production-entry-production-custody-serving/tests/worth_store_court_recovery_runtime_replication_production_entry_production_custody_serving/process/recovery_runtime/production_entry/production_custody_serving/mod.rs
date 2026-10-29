//! Production recovery and Store construction, with no certification pause.
//! The feature is used only to create authoritative release/tier fixtures.

#[allow(unused_imports)] use crate::production_entry::production_custody_serving::*;
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world, worth_store_testkit_recovery_runtime::production_entry::release_reopen};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
use {worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution};

#[test]
fn production_release_custody_opens_serving_after_fresh_recovery() {
    let (world, _, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1024);
    checkpoint(&world, 0xc1);
    recover_and_open(world);
}

#[test]
fn production_two_distinct_releases_open_serving_after_carryforward() {
    let (world, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations::world();
    checkpoint(&world, 0xc2);
    recover_and_open(world);
}

#[test]
fn production_mixed_failed_ingest_and_release_open_serving() {
    let world = crate::production_entry::release_reopen::mixed_failed_ingest::world();
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
    let (world, _, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_tier_world(1024);
    checkpoint(&world, 0xc6);
    recover_and_open(world);
}
