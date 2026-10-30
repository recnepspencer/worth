//! Destination-first release of a reused publication: the drop invalidates the
//! directory watermark, publishes a replacement binding in the same transition,
//! and the surviving source stays readable through ordinary catalog reads
//! before and after a checkpoint and fresh C8 reopen.

pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {
    worth_store::physical_runtime::BlobReclaimDeferral,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    crate::production_entry::release_reopen::shared_reuse_custody::assert_absent,
    crate::production_entry::release_reopen::shared_reuse_custody::assert_retired,
    crate::production_entry::release_reopen::shared_reuse_custody::checkpoint,
    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process,
    crate::production_entry::release_reopen::shared_reuse_custody::read_exact,
    crate::production_entry::release_reopen::shared_reuse_custody::recover_serving,
    crate::production_entry::release_reopen::shared_reuse_custody::release,
    crate::production_entry::release_reopen::shared_reuse_custody::release_request,
    crate::production_entry::release_reopen::shared_reuse_custody::selected_marker,
    crate::production_entry::release_reopen::shared_reuse_custody::shared_format,
    crate::production_entry::release_reopen::shared_reuse_custody::world,
    crate::production_entry::release_reopen::shared_reuse_custody::SCOPE_NAME,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
};

pub(crate) fn run() {
    let format = shared_format();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_operation_budget(
        "c11-shared-destination-first-c8",
        format,
        512,
        crate::production_entry::release_reopen::shared_reuse_custody::world::release_operation_budget(),
    )
    .expect("genuine shared-reuse recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let payload = crate::production_entry::release_reopen::shared_reuse_custody::world::payload();
    let original =
        crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
            &world, &scope, &payload,
        );
    let original_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xe1; 32]);
    let destination =
        crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
            &world, &scope, &payload,
        );
    let destination_marker = selected_marker(world.serving());
    assert_eq!(
        read_exact(world.serving(), &scope, destination, &payload),
        6
    );

    // A held reader defers before any effect; the retry then completes exactly.
    let held = world.serving().records().unwrap();
    assert!(matches!(
        world.serving().blobs().unwrap().reclaim(release_request(
            world.serving(),
            world.placement(),
            destination,
            destination_marker,
        )),
        Err(worth_store::physical_runtime::BlobReclaimFailure::Deferred(
            worth_store::physical_runtime::BlobReclaimDeferral::ProtectedReader
        ))
    ));
    drop(held);
    assert_eq!(selected_marker(world.serving()), destination_marker);

    // The destination is the directory watermark, so this drop must publish
    // the replacement binding that retains the shared catalog root.
    let first = release(
        world.serving(),
        world.placement(),
        destination,
        destination_marker,
    );
    assert_retired(&first);
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert!(first
        .dropped_records()
        .contains(&destination_marker.record()));
    assert!(!first.dropped_records().contains(&original_marker.record()));
    assert_eq!(first.remaining_payload_records(), 0);
    assert_no_latest(world.serving());
    assert_absent(world.serving(), &scope, destination.object().bytes());
    assert_eq!(read_exact(world.serving(), &scope, original, &payload), 0);
    checkpoint(world.serving(), [0xe2; 32]);
    let producer_policy = world.serving().residency_observation().admitted_policy();
    let recovery_bytes = producer_policy
        .operation_bytes()
        .min(producer_policy.scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::assert_reopens(
        &root,
        destination.object().bytes(),
        original.object().bytes(),
        recovery_bytes,
        crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::ExpectedReopen::Surviving { reuse_reads: 0 },
    );
    let serving = recover_serving(&root, format, recovery_bytes);
    assert_no_latest(&serving);
    assert_absent(&serving, &scope, destination.object().bytes());
    assert_eq!(read_exact(&serving, &scope, original, &payload), 0);

    // The repeated destination release is proven to have no effect.
    let growth = serving.certification_charged_growth_bytes();
    let repeated = release(
        &serving,
        crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
            format,
        ),
        destination,
        destination_marker,
    );
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.bytes_released(), 0);
    assert_eq!(serving.certification_charged_growth_bytes(), growth);

    let source = release(
        &serving,
        crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
            format,
        ),
        original,
        original_marker,
    );
    assert_retired(&source);
    assert!(source.dropped_records().contains(&original_marker.record()));
    assert_eq!(source.remaining_payload_records(), 0);
    assert_absent(&serving, &scope, original.object().bytes());
    checkpoint(&serving, [0xe3; 32]);
    serving.close();

    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::assert_reopens(
        &root,
        destination.object().bytes(),
        original.object().bytes(),
        recovery_bytes,
        crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::ExpectedReopen::BothInvisible,
    );
}

pub(crate) fn assert_no_latest(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        serving
            .certification_selected_latest_blob_publication()
            .unwrap(),
        None,
        "dropping the latest publication clears the selected latest hint",
    );
}
