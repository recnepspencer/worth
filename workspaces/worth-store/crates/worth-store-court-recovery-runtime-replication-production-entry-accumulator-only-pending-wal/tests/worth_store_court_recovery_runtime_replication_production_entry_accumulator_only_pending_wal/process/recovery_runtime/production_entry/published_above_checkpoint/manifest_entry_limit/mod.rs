//! Ordering the retirements above the checkpoint charges what every root
//! step among them declares it wrote. A request that admits one manifest
//! entry too few must be told so, with the count the refused charge reached,
//! and the same request with that one entry more recovers.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::manifest_entry_limit::*;
use {worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::published_above_checkpoint};
use {worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld};
use {worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

/// No release walked the history: the walk runs only for the retired pages.
#[test]
fn a_walk_for_retired_pages_one_manifest_entry_short_reports_that_limit() {
    let stage = "published above the checkpoint";
    let world =
        pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle);
    assert_one_entry_short_reports_the_limit(&world, IDLE_NEED, stage);
    let serving = serve_under(&world, IDLE_NEED, stage);
    world.assert_objects_read_back(&serving);
    serving.close();
}

#[test]
fn a_historical_release_one_manifest_entry_short_reports_that_limit() {
    let stage = "released above the checkpoint";
    let world = released_world();
    assert_one_entry_short_reports_the_limit(&world, RELEASED_NEED, stage);
    serve_under(&world, RELEASED_NEED, stage).close();
}

#[test]
fn an_ordered_release_above_a_head_checkpoint_one_manifest_entry_short_reports_that_limit() {
    let stage = "ordered release above a head checkpoint";
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::ordered_release_above_a_head_checkpoint();
    assert_one_entry_short_reports_the_limit(&world, ORDERED_NEED, stage);
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_manifest_entries(world.root(), ORDERED_NEED),
    );
    assert!(
        matches!(outcome, worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_)),
        "{stage}: {ORDERED_NEED} entries must recover: {outcome:?}",
    );
}
