//! A selected FailedIngest cleanup remains benign beside a later, independent
//! published-generation release on the same Store media.

use super::*;
use worth_store::physical_runtime::BlobTerminalLimits;

const CHUNK: usize = 64 << 10;

pub(crate) fn run() {
    let world = world();
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    two_batch::recover(root);
}

pub(crate) fn world() -> PhysicalResidencyStoreWorld {
    let world = initialized_recovery_world("mixed-failed-and-released");
    let failed_scope = admitted_blob_scope("c11.recovery.mixed.failed.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let read_limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let failed_object = blobs.issue_object_id(read_limits).expect("failed object");
    let failed_declaration = BlobIngestDeclaration::new(
        failed_object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &failed_scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
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
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobTerminalLimits::new(NonZeroU64::new(128).unwrap()),
        )
        .expect("failed-ingest terminal");
    let failed = blobs
        .reclaim(BlobReclaimRequest::abandoned(
            token,
            &failed_scope,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(128).unwrap(),
                NonZeroU64::new(8 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("failed-ingest reclaim")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(failed.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(failed.dropped_records().len(), 1);
    drop(blobs);
    checkpoint(&world, [0xc1; 32]);

    let released_scope = admitted_blob_scope("c11.recovery.mixed.released.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let object = blobs.issue_object_id(read_limits).expect("released object");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &released_scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read_limits)
        .expect("begin published ingest");
    ingest.push(&[0x51; CHUNK]).expect("published first chunk");
    ingest.push(&[0x52; CHUNK]).expect("published second chunk");
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("independent publication failed: {failure:?}"),
    };
    drop(blobs);
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected independent publication");
    let record = marker.record();
    let proof = AdmittedBlobReleaseProof::certification_admit(
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
        .reclaim(BlobReclaimRequest::released(
            proof,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(1024).unwrap(),
                NonZeroU64::new(64 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("released-generation reclaim")
        .wait()
        .expect("released-generation drop");
    assert_eq!(released.disposition(), BlobReclaimDisposition::Dropped);
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

fn checkpoint(world: &PhysicalResidencyStoreWorld, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("mixed-history checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

/// Both histories keep every control record selected: the failed-ingest
/// manifest and descriptor, and the released manifest, reservation, and
/// descriptor.
fn assert_controls_selected(serving: &ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(64).unwrap())
                .with_payload_limit(RecordByteLimit::new(8192).unwrap()),
        )
        .unwrap();
    let mut scratch = [0; 64 << 10];
    let mut failed_manifest = None;
    let mut failed_descriptor = None;
    let mut released_manifest = None;
    let mut released_descriptor = None;
    let mut reservations = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let visible = batch.records()[index].record_id();
            let record =
                PersistedRecordIdentity::new(visible.allocation_epoch(), visible.ordinal())
                    .expect("serving record has persisted identity");
            match decode_blob_record(bytes) {
                Ok(BlobRecordV1::DropSetManifest(_) | BlobRecordV1::DropSetManifestV2(_)) => {
                    failed_manifest = Some(record)
                }
                Ok(BlobRecordV1::ReclaimDescriptor(_)) => failed_descriptor = Some(record),
                Ok(BlobRecordV1::ReclaimDescriptorV2(value))
                    if value.source_kind() == BlobReclaimSourceKind::FailedIngest =>
                {
                    failed_descriptor = Some(record)
                }
                Ok(BlobRecordV1::DropSetManifestV3(_)) => released_manifest = Some(record),
                Ok(BlobRecordV1::ReclaimDescriptorV3(_)) => released_descriptor = Some(record),
                Ok(BlobRecordV1::OriginalDropReserved(value)) => {
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
