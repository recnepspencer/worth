//! One genuine partial release populates the custody used by the policy proof.

pub(crate) use {worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::ledger_preparation::world};

pub(crate) fn checkpointed_partial_release_world() -> PhysicalResidencyStoreWorld {
    let (world, release, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
    assert_eq!(release.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert_eq!(release.dropped_records().len(), 1);
    assert!(release.remaining_payload_records() > 0);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xb7; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("partial-release checkpoint must admit")
    };
    assert!(
        matches!(handle.wait(), worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)),
        "partial-release custody checkpoint must complete"
    );
    world
}