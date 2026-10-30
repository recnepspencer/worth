//! The first killed producer, in the order it runs: the baseline under its
//! checkpoint, the publications above it, and the tail it is killed in.

pub use worth_store_physical_format::IndexedThroughBlobPublication;
pub use {
    crate::pending_wal_world::descriptor_park::park_at_descriptor_wal,
    crate::production_entry::blob_scope_admission::admitted_blob_scope,
    crate::production_entry::pending_wal_world::first::chunk,
    crate::production_entry::pending_wal_world::first::launch::first,
    crate::production_entry::pending_wal_world::first::objects_path,
    crate::production_entry::pending_wal_world::first::World,
    crate::production_entry::pending_wal_world::CHUNK,
    crate::production_entry::pending_wal_world::WAL_SEGMENT_BYTES_ENV, std::fs,
    std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobTerminalLimits,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::RecordAppendBatch, worth_store_blob_chunks::BlobChunkSize,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub use {
    worth_store::physical_runtime::BlobObjectId,
    worth_store::physical_runtime::PublishedBlobGeneration,
};

pub fn child(marker: &Path, shape: World) {
    let world = baseline(shape);
    report_progress(marker, "baseline checkpoint");
    let (objects, first) = publish(&world, shape, marker);
    std::fs::write(objects_path(marker), objects).unwrap();
    let root_path = world.root().to_string_lossy().into_owned();
    if shape.releases_first_object() {
        park_release(&world, &first, marker, root_path.as_bytes());
    } else {
        park_idle(marker, root_path.as_bytes());
    }
}

/// The first object the child published, as its release names it.
pub struct FirstPublication {
    pub object: BlobObjectId,
    pub published: PublishedBlobGeneration,
    pub publication: IndexedThroughBlobPublication,
}

/// A served store whose baseline record is under the baseline checkpoint.
pub fn baseline(shape: World) -> PhysicalResidencyStoreWorld {
    let world = match std::env::var(WAL_SEGMENT_BYTES_ENV) {
        Ok(value) => worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
            "c11-pending-v3",
            std::num::NonZeroU64::new(value.parse().expect("bounded WAL segment size"))
                .expect("nonzero WAL segment size"),
        ),
        Err(std::env::VarError::NotPresent) => {
            worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c11-pending-v3")
        }
        Err(error) => panic!("invalid WAL segment environment: {error}"),
    }
    .expect("first pending WAL producer");
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xa0; 32]),
        )
        .unwrap();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"c11-pending-v3-frontier".as_slice(),
            ])
            .unwrap(),
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    else {
        panic!("baseline C5 frontier record must prepare")
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
    if shape == crate::production_entry::pending_wal_world::first::World::FailedIngestControl {
        create_failed_ingest_control(&world);
    }
    checkpoint(&world, 0xa1, "baseline NoRelease");
    world
}

/// Publishes every object of the shape above the baseline checkpoint. Returns
/// their identities in publication order and the first publication.
pub fn publish(
    world: &PhysicalResidencyStoreWorld,
    shape: World,
    marker: &Path,
) -> (Vec<u8>, FirstPublication) {
    let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
    let blobs = world.serving().blobs().unwrap();
    let read =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let mut objects = Vec::new();
    let mut first = None;
    for (index, chunks) in shape.objects().iter().enumerate() {
        let object = blobs.issue_object_id(read).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            (chunks * CHUNK) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
            .unwrap();
        for ordinal in 0..*chunks {
            ingest.push(&chunk(index, ordinal)).unwrap();
            report_progress(marker, &format!("object {index}, chunk {ordinal}"));
            if shape == crate::production_entry::pending_wal_world::first::World::ResumeFrontier
                && ordinal == 0
            {
                ingest.checkpoint().unwrap();
                report_progress(marker, &format!("object {index}, explicit resume frontier"));
            }
        }
        let published = match ingest.finish() {
            Ok(published)
            | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending {
                published,
                ..
            }) => published,
            Err(failure) => panic!("publication before the kill: {failure:?}"),
        };
        assert_eq!(published.object(), object);
        assert_eq!(published.generation().sequence(), 1);
        report_progress(marker, &format!("object {index}, publication"));
        objects.extend(object.bytes());
        if first.is_none() {
            let publication = world
                .serving()
                .certification_selected_latest_blob_publication()
                .unwrap()
                .unwrap();
            assert!(publication.root_generation() > 1);
            first = Some(FirstPublication {
                object,
                published,
                publication,
            });
        }
    }
    (objects, first.expect("first published object"))
}

// A parent watchdog observes completed production operations, never a timer
// heartbeat. This file is outside the Store and carries no recovery evidence.
pub fn report_progress(marker: &Path, stage: &str) {
    let pending = marker.with_extension("progress-pending");
    std::fs::write(&pending, stage).unwrap();
    std::fs::rename(pending, marker.with_extension("progress")).unwrap();
}

/// Announces the root and idles until the parent kills the process.
pub fn park_idle(marker: &Path, root_path: &[u8]) -> ! {
    let pending = marker.with_extension("pending");
    std::fs::write(&pending, root_path).unwrap();
    std::fs::rename(pending, marker).unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Parks a release of the first object at its durable descriptor WAL.
pub fn park_release(
    world: &PhysicalResidencyStoreWorld,
    first: &FirstPublication,
    marker: &Path,
    root_path: &[u8],
) {
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        first.object.bytes(),
        first.published.generation().sequence(),
        first.publication.record().allocation_epoch(),
        first.publication.record().ordinal(),
        first.publication.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(32 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    park_at_descriptor_wal(world.serving(), request, marker, root_path);
}

pub fn checkpoint(world: &PhysicalResidencyStoreWorld, key: u8, stage: &str) {
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("{stage} checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub fn create_failed_ingest_control(world: &PhysicalResidencyStoreWorld) {
    let scope = admitted_blob_scope("c11.recovery.mixed-pending.failed.scope");
    let blobs = world.serving().blobs().expect("failed-ingest blob owner");
    let read =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read).expect("failed object");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
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
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
            worth_store::physical_runtime::BlobTerminalLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
            ),
        )
        .expect("failed-ingest abandonment");
    let receipt = blobs
        .reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::abandoned(
                token,
                &scope,
                world.placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(
                    120_000,
                )
                .unwrap(),
                worth_store::physical_runtime::BlobReclaimLimits::new(
                    std::num::NonZeroU64::new(128).unwrap(),
                    std::num::NonZeroU64::new(8 << 20).unwrap(),
                    std::num::NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ),
        )
        .expect("failed-ingest reclaim admission")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(receipt.dropped_records().len(), 1);
}
