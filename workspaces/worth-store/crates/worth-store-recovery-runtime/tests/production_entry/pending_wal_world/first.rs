//! First real killed V3 producer, optionally using a bounded WAL segment size.

use super::*;
use worth_store::physical_runtime::{BlobReclaimDisposition, BlobTerminalLimits};

/// What the first child leaves selected under the release it parks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum World {
    /// One two-chunk object, published above the baseline checkpoint.
    TwoChunks,
    /// The two-chunk object, after a failed ingest left its drop controls.
    FailedIngestControl,
    /// One object with two resume frontiers: the ingest checkpoints one at
    /// its first chunk and another by itself 64 chunks later, one chunk
    /// before its end. A checkpoint covers the ingest: above a checkpoint,
    /// recovery blocks on the inline pages an ingest this long allocated and
    /// retired (`PageAdmission`), which is not what this world is about.
    ResumeFrontier,
}

impl World {
    const ALL: [Self; 3] = [
        Self::TwoChunks,
        Self::FailedIngestControl,
        Self::ResumeFrontier,
    ];

    /// The child role that builds this world.
    pub(super) const fn role(self) -> &'static str {
        match self {
            Self::TwoChunks => "first",
            Self::FailedIngestControl => "mixed-first",
            Self::ResumeFrontier => "frontier-first",
        }
    }

    pub(super) fn of_role(role: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|world| world.role() == role)
    }

    /// The first world of the root a later child was started on.
    pub(super) fn of_child_root() -> Self {
        std::env::var(FIRST_WORLD_ENV)
            .ok()
            .and_then(|role| Self::of_role(&role))
            .expect("first world of the child root")
    }

    const fn chunks(self) -> usize {
        match self {
            Self::TwoChunks | Self::FailedIngestControl => 2,
            Self::ResumeFrontier => 66,
        }
    }

    pub(super) fn recovery_request(
        self,
        root: &Path,
    ) -> worth_store_recovery_runtime::PhysicalRecoveryOpenRequest {
        match self {
            Self::TwoChunks | Self::FailedIngestControl => {
                super::super::certified_release_serving::request(root)
            }
            Self::ResumeFrontier => {
                super::super::certified_release_serving::request_for_long_ingest(root)
            }
        }
    }
}

/// Distinct whole chunks, so every one is its own selected record. The first
/// two are the bytes of the two-chunk world.
fn chunk(ordinal: usize) -> Vec<u8> {
    let mut chunk = vec![if ordinal % 2 == 0 { 0x83 } else { 0x94 }; CHUNK];
    let salt = ((ordinal / 2) as u64).to_le_bytes();
    for (byte, salt) in chunk.iter_mut().zip(salt) {
        *byte ^= salt;
    }
    chunk
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

pub(super) fn child(marker: &Path, shape: World) {
    let chunks = shape.chunks();
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
    let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
    let blobs = world.serving().blobs().unwrap();
    let read = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
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
    for ordinal in 0..chunks {
        ingest.push(&chunk(ordinal)).unwrap();
        if shape == World::ResumeFrontier && ordinal == 0 {
            ingest.checkpoint().unwrap();
        }
    }
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
    if shape == World::ResumeFrontier {
        checkpoint(&world, 0xa2, "published frontier object");
    }
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
