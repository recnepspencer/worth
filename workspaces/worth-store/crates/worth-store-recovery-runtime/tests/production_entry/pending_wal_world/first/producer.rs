//! The first killed producer, in the order it runs: the baseline under its
//! checkpoint, the publications above it, and the tail it is killed in.

use super::*;
use worth_store::physical_runtime::{BlobObjectId, PublishedBlobGeneration};
use worth_store_physical_format::IndexedThroughBlobPublication;

pub(in super::super) fn child(marker: &Path, shape: World) {
    let world = baseline(shape);
    let (objects, first) = publish(&world, shape);
    fs::write(objects_path(marker), objects).unwrap();
    let root_path = world.root().to_string_lossy().into_owned();
    if shape.releases_first_object() {
        park_release(&world, &first, marker, root_path.as_bytes());
    } else {
        park_idle(marker, root_path.as_bytes());
    }
}

/// The first object the child published, as its release names it.
struct FirstPublication {
    object: BlobObjectId,
    published: PublishedBlobGeneration,
    publication: IndexedThroughBlobPublication,
}

/// A served store whose baseline record is under the baseline checkpoint.
fn baseline(shape: World) -> PhysicalResidencyStoreWorld {
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
    if shape == World::FailedIngestControl {
        create_failed_ingest_control(&world);
    }
    checkpoint(&world, 0xa1, "baseline NoRelease");
    world
}

/// Publishes every object of the shape above the baseline checkpoint. Returns
/// their identities in publication order and the first publication.
fn publish(world: &PhysicalResidencyStoreWorld, shape: World) -> (Vec<u8>, FirstPublication) {
    let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
    let blobs = world.serving().blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let mut objects = Vec::new();
    let mut first = None;
    for (index, chunks) in shape.objects().iter().enumerate() {
        let object = blobs.issue_object_id(read).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
            (chunks * CHUNK) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, world.placement(), CHUNK as u64, read)
            .unwrap();
        for ordinal in 0..*chunks {
            ingest.push(&chunk(index, ordinal)).unwrap();
            if shape == World::ResumeFrontier && ordinal == 0 {
                ingest.checkpoint().unwrap();
            }
        }
        let published = match ingest.finish() {
            Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
                published
            }
            Err(failure) => panic!("publication before the kill: {failure:?}"),
        };
        assert_eq!(published.object(), object);
        assert_eq!(published.generation().sequence(), 1);
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

/// Announces the root and idles until the parent kills the process.
fn park_idle(marker: &Path, root_path: &[u8]) -> ! {
    let pending = marker.with_extension("pending");
    fs::write(&pending, root_path).unwrap();
    fs::rename(pending, marker).unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Parks a release of the first object at its durable descriptor WAL.
fn park_release(
    world: &PhysicalResidencyStoreWorld,
    first: &FirstPublication,
    marker: &Path,
    root_path: &[u8],
) {
    let proof = AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        first.object.bytes(),
        first.published.generation().sequence(),
        first.publication.record().allocation_epoch(),
        first.publication.record().ordinal(),
        first.publication.encoded_digest(),
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
    park_at_descriptor_wal(world.serving(), request, marker, root_path);
}

fn checkpoint(world: &PhysicalResidencyStoreWorld, key: u8, stage: &str) {
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("{stage} checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
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
