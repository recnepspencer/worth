//! Releasing a lagging directory watermark while a newer publication survives:
//! the replacement binding drops only the invalid watermark, keeps the newer
//! latest hint, and leaves catalog reads stale until an ordinary rebuild.

pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use worth_store_physical_format::IndexedThroughBlobPublication;
pub(crate) use worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration;
pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::PhysicalLayoutDenial,
    worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    crate::production_entry::release_reopen::shared_reuse_custody::assert_absent,
    crate::production_entry::release_reopen::shared_reuse_custody::assert_retired,
    crate::production_entry::release_reopen::shared_reuse_custody::checkpoint,
    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process,
    crate::production_entry::release_reopen::shared_reuse_custody::read_exact,
    crate::production_entry::release_reopen::shared_reuse_custody::recover_custody,
    crate::production_entry::release_reopen::shared_reuse_custody::recover_serving,
    crate::production_entry::release_reopen::shared_reuse_custody::release,
    crate::production_entry::release_reopen::shared_reuse_custody::selected_marker,
    crate::production_entry::release_reopen::shared_reuse_custody::shared_format,
    crate::production_entry::release_reopen::shared_reuse_custody::world,
    crate::production_entry::release_reopen::shared_reuse_custody::CHUNK,
    crate::production_entry::release_reopen::shared_reuse_custody::SCOPE_NAME,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
};

/// Below the 4 MiB catalog source-validation grant, so the newer publication
/// becomes durable while its derived catalog is denied.
pub(crate) const MAINTENANCE_LOW_BYTES: u64 = 1 << 20;

pub(crate) fn run() {
    let format = shared_format();
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_format_and_manifest_capacity(
            "c11-lagging-watermark-release",
            format,
            512,
        )
        .expect("genuine KiB64 recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let indexed_payload = vec![0x41; CHUNK + 17];
    let indexed = crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
        &world,
        &scope,
        &indexed_payload,
    );
    let indexed_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xf1; 32]);
    let producer_policy = world.serving().residency_observation().admitted_policy();
    let recovery_bytes = producer_policy
        .operation_bytes()
        .min(producer_policy.scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    // Only the Maintenance scope differs: the newer publication is durable,
    // but its catalog validation is denied, so the watermark lags.
    let policy = low_maintenance_policy(format);
    let configuration =
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::for_record_format(
            format.declaration(),
        )
        .with_residency_policy(policy)
        .expect("policy admitted for the configured format");
    let seal = recover_custody(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(
        &root,
        recovery_bytes,
        configuration,
    ));
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal_format_and_policy(
        &root, seal, format, policy,
    );
    let newer = publish_unindexed(
        &serving,
        format,
        &scope,
        &crate::production_entry::release_reopen::shared_reuse_custody::world::payload(),
    );
    let newer_marker = selected_marker(&serving);
    assert_ne!(newer_marker, indexed_marker);
    assert_stale(&serving, &scope, newer, newer_marker, Some(indexed_marker));
    // Checkpoint compaction also needs Maintenance, so C8 replays this
    // publication from the WAL tail.
    serving.close();

    let serving = recover_serving(&root, format, recovery_bytes);
    assert_stale(&serving, &scope, newer, newer_marker, Some(indexed_marker));
    let receipt = release(
        &serving,
        crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
            format,
        ),
        indexed,
        indexed_marker,
    );
    assert_retired(&receipt);
    assert!(receipt.dropped_records().contains(&indexed_marker.record()));
    assert!(!receipt.dropped_records().contains(&newer_marker.record()));
    // The replacement binding clears only the invalid watermark; the newer
    // latest hint survives, so reads stay stale instead of scanning.
    assert_eq!(selected_marker(&serving), newer_marker);
    assert_stale(&serving, &scope, newer, newer_marker, None);
    assert!(!serving.certification_selected_layout_record(indexed_marker.record()));

    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
            worth_store::physical_runtime::LayoutRebuildLimits::new(
                std::num::NonZeroU64::new(1024).unwrap(),
                std::num::NonZeroU64::new(1024).unwrap(),
            ),
            crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
                format,
            ),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .expect("ordinary rebuild from the selected authoritative closure");
    assert_eq!(rebuilt.source(), newer_marker);
    assert_absent(&serving, &scope, indexed.object().bytes());
    assert_eq!(
        read_exact(
            &serving,
            &scope,
            newer,
            &crate::production_entry::release_reopen::shared_reuse_custody::world::payload()
        ),
        0
    );
    checkpoint(&serving, [0xf3; 32]);
    serving.close();

    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::assert_reopens(
        &root,
        indexed.object().bytes(),
        newer.object().bytes(),
        recovery_bytes,
        crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::ExpectedReopen::Surviving { reuse_reads: 0 },
    );
}

pub(crate) fn publish_unindexed(
    serving: &ServingPhysicalRuntime,
    format: AdmittedPhysicalRecordFormat,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            declaration,
            crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
                format,
            ),
            (CHUNK / 2) as u64,
            limits,
        )
        .expect("physical ingest admits before the maintenance denial");
    for piece in payload.chunks(CHUNK / 2) {
        ingest.push(piece).expect("physical chunk append");
    }
    match ingest.finish() {
        Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending {
            published,
            cause: worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure::FullAuthority(_),
        }) => published,
        other => panic!("catalog validation must deny after durable publication: {other:?}"),
    }
}

pub(crate) fn assert_stale(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    newer: PublishedBlobGeneration,
    latest: IndexedThroughBlobPublication,
    indexed: Option<IndexedThroughBlobPublication>,
) {
    let resolved = serving.blobs().unwrap().resolve_publication(
        newer.object().bytes(),
        newer.generation().sequence(),
        scope,
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap()),
    );
    assert!(
        matches!(
            &resolved,
            Err(worth_store::physical_runtime::BlobReadOpenFailure::Layout(worth_store::physical_runtime::PhysicalLayoutDenial::IndexStaleRequiresRebuild {
                latest: observed_latest,
                indexed: observed_indexed,
            })) if *observed_latest == latest && *observed_indexed == indexed
        ),
        "stale catalog must deny without an authoritative scan: {resolved:?}"
    );
}

pub(crate) fn low_maintenance_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let canonical =
        worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| std::num::NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let mut builder = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
        .progress_headroom_bytes(64 << 10);
    for scope in [
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
    ] {
        let value = match scope {
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance => {
                MAINTENANCE_LOW_BYTES
            }
            _ => canonical.scope_bytes(scope),
        };
        builder = builder.scope_bytes(scope, bytes(value));
    }
    for kind in [
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead,
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch,
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("only the Maintenance scope differs")
}
