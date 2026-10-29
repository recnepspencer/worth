//! A selected FailedIngest cleanup remains benign beside a later, independent
//! published-generation release on the same Store media.

pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_physical_format::BlobReclaimSourceKind, worth_store_physical_format::BlobRecordV1, std::num::NonZeroU16, std::num::NonZeroU64, worth_store_physical_format::PersistedRecordIdentity, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_physical_format::decode_blob_record, worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world, worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates, worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_batch};
pub(crate) use {worth_store::physical_runtime::BlobTerminalLimits};

pub(crate) const CHUNK: usize = 64 << 10;

pub(crate) fn run() {
    let world = world();
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_batch::recover(root);
}

pub(crate) fn world() -> PhysicalResidencyStoreWorld {
    let world = initialized_recovery_world("mixed-failed-and-released");
    let failed_scope = admitted_blob_scope("c11.recovery.mixed.failed.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let failed_object = blobs.issue_object_id(read_limits).expect("failed object");
    let failed_declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        failed_object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &failed_scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            failed_declaration,
            world.placement(),
            CHUNK as u64,
            read_limits,
        )
        .expect("begin failed ingest");
    ingest.push(&[0x41; CHUNK]).expect("failed-ingest chunk");
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(
            token,
            &failed_scope,
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            worth_store::physical_runtime::BlobTerminalLimits::new(std::num::NonZeroU64::new(128).unwrap()),
        )
        .expect("failed-ingest terminal");
    let failed = blobs
        .reclaim(worth_store::physical_runtime::BlobReclaimRequest::abandoned(
            token,
            &failed_scope,
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
                std::num::NonZeroU64::new(8 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("failed-ingest reclaim")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(failed.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert_eq!(failed.dropped_records().len(), 1);
    checkpoint(&world, [0xc1; 32]);

    let released_scope = admitted_blob_scope("c11.recovery.mixed.released.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let object = blobs.issue_object_id(read_limits).expect("released object");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &released_scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read_limits)
        .expect("begin published ingest");
    ingest.push(&[0x51; CHUNK]).expect("published first chunk");
    ingest.push(&[0x52; CHUNK]).expect("published second chunk");
    let published = match ingest.finish() {
        Ok(published) | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("independent publication failed: {failure:?}"),
    };
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected independent publication");
    let record = marker.record();
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0xc2; 32],
    )
    .unwrap();
    let released = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(1024).unwrap(),
                std::num::NonZeroU64::new(64 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("released-generation reclaim")
        .wait()
        .expect("released-generation drop");
    assert_eq!(released.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert!(released.dropped_records().contains(&record));
    let (batches, accumulator) = selected_release_certificates(&world);
    assert_eq!(batches.len(), 1);
    assert_eq!(
        accumulator.base().tip(),
        batches[0].tip_provenance().unwrap()
    );
    assert_controls_selected(world.serving());
    world
}

pub(crate) fn checkpoint(world: &PhysicalResidencyStoreWorld, key: [u8; 32]) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("mixed-history checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

/// Both histories keep every control record selected: the failed-ingest
/// manifest and descriptor, and the released manifest, reservation, and
/// descriptor.
pub(crate) fn assert_controls_selected(serving: &ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(64).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(8192).unwrap()),
        )
        .unwrap();
    let mut scratch = [0; 64 << 10];
    let mut failed_manifest = None;
    let mut failed_descriptor = None;
    let mut released_manifest = None;
    let mut released_descriptor = None;
    let mut reservations = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let visible = batch.records()[index].record_id();
            let record =
                worth_store_physical_format::PersistedRecordIdentity::new(visible.allocation_epoch(), visible.ordinal())
                    .expect("serving record has persisted identity");
            match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifest(_) | worth_store_physical_format::BlobRecordV1::DropSetManifestV2(_)) => {
                    failed_manifest = Some(record)
                }
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(_)) => failed_descriptor = Some(record),
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV2(value))
                    if value.source_kind() == worth_store_physical_format::BlobReclaimSourceKind::FailedIngest =>
                {
                    failed_descriptor = Some(record)
                }
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV3(_)) => released_manifest = Some(record),
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV3(_)) => released_descriptor = Some(record),
                Ok(worth_store_physical_format::BlobRecordV1::OriginalDropReserved(value)) => {
                    reservations.push(value.manifest_record())
                }
                _ => {}
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    let released_manifest = released_manifest.expect("selected released manifest");
    assert!(
        reservations.contains(&released_manifest),
        "selected released reservation"
    );
    assert!(failed_manifest.is_some(), "selected failed-ingest manifest");
    assert!(
        failed_descriptor.is_some(),
        "selected failed-ingest descriptor"
    );
    assert!(
        released_descriptor.is_some(),
        "selected released descriptor"
    );
}