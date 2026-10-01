//! First real killed V3 producer, optionally using a bounded WAL segment size.

use super::*;
use worth_store::physical_runtime::{BlobReclaimDisposition, BlobTerminalLimits};

pub(super) fn child(marker: &Path, failed_ingest_control: bool) {
    let world = match std::env::var(WAL_SEGMENT_BYTES_ENV) {
        Ok(value) => PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
            "c11-pending-v3",
            NonZeroU64::new(value.parse().expect("bounded WAL segment size"))
                .expect("nonzero WAL segment size"),
        ),
        Err(std::env::VarError::NotPresent) => {
            PhysicalResidencyStoreWorld::initialize_for_recovery("c11-pending-v3")
        }
        Err(error) => panic!("invalid WAL segment environment: {error}"),
    }
    .expect("first pending WAL producer");
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xa0; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"c11-pending-v3-frontier".as_slice()]).unwrap(),
                world.placement(),
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("baseline C5 frontier record must prepare")
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
    if failed_ingest_control {
        create_failed_ingest_control(&world);
    }
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xa1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("baseline NoRelease checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
    let blobs = world.serving().blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .unwrap();
    ingest.push(&vec![0x83; CHUNK]).unwrap();
    ingest.push(&vec![0x94; CHUNK]).unwrap();
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("publication before pending descriptor: {failure:?}"),
    };
    drop(blobs);
    let publication = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    assert_eq!(published.object(), object);
    assert_eq!(published.generation().sequence(), 1);
    assert!(publication.root_generation() > 1);
    let proof = AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        publication.record().allocation_epoch(),
        publication.record().ordinal(),
        publication.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let serving = world.serving();
    let root_path = world.root().to_string_lossy().into_owned();
    park_at_descriptor_wal(serving, request, marker, root_path.as_bytes());
}

fn create_failed_ingest_control(world: &PhysicalResidencyStoreWorld) {
    let scope = admitted_blob_scope("c11.recovery.mixed-pending.failed.scope");
    let blobs = world.serving().blobs().expect("failed-ingest blob owner");
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).expect("failed object");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
        .expect("begin failed ingest");
    ingest.push(&[0x47; CHUNK]).expect("failed-ingest chunk");
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(
            token,
            &scope,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            BlobTerminalLimits::new(NonZeroU64::new(128).unwrap()),
        )
        .expect("failed-ingest abandonment");
    let receipt = blobs
        .reclaim(BlobReclaimRequest::abandoned(
            token,
            &scope,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(128).unwrap(),
                NonZeroU64::new(8 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("failed-ingest reclaim admission")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
}
