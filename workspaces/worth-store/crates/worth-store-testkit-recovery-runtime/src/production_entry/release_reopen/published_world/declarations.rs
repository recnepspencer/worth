//! One genuine selected publication and owner-issued release proof, before reclaim.

pub use {
    crate::production_entry::blob_scope_admission::admitted_blob_scope, std::num::NonZeroU64,
    worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_blob_chunks::BlobChunkSize, worth_store_physical_format::PersistedRecordIdentity,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
};

pub fn create(
    activate_tier: bool,
    wal_segment_bytes: Option<NonZeroU64>,
    recovery_scope_bytes: Option<NonZeroU64>,
) -> (
    PhysicalResidencyStoreWorld,
    AdmittedBlobReleaseProof,
    PersistedRecordIdentity,
) {
    assert!(wal_segment_bytes.is_none() || recovery_scope_bytes.is_none());
    const CHUNK: usize = 64 << 10;
    let world = match (wal_segment_bytes, recovery_scope_bytes) {
        (Some(bytes), None) => {
            worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
                "release-checkpoint-reopen",
                bytes,
            )
        }
        (None, Some(bytes)) => {
            worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_recovery_scope(
                "release-checkpoint-reopen",
                worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                    worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
                ),
                4,
                bytes,
            )
        }
        (None, None) => {
            worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("release-checkpoint-reopen")
        }
        (Some(_), Some(_)) => unreachable!("the fixture admits one policy override"),
    }
    .expect("initialize release world");
    if activate_tier {
        world
            .serving()
            .certification_activate_tier_epoch(world.placement())
            .expect("typed one-time tier activation before release");
    }
    let scope = admitted_blob_scope("c11.recovery.release.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(read_limits).expect("object identity");
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
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read_limits)
        .expect("begin ingest");
    ingest.push(&vec![0x31; CHUNK]).expect("first chunk");
    ingest.push(&vec![0x52; CHUNK]).expect("second chunk");
    let published = match ingest.finish() {
        Ok(published)
        | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending {
            published,
            ..
        }) => published,
        Err(failure) => panic!("publication failed: {failure:?}"),
    };
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication");
    let record = marker.record();
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    (world, proof, record)
}
