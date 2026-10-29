pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDeferral, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalPageSizeClass, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PublishedBlobGeneration, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {crate::physical_blob_journeys::blob_ingest_process::observed_without_damage, crate::physical_blob_journeys::blob_ingest_process::selected_root};
pub(crate) use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

pub(crate) const CHUNK: usize = 64 << 10;
pub(crate) const SHARED_CHUNK: usize = 256 << 10;

pub(crate) fn publish_one(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(SHARED_CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            declaration,
            shared_placement(),
            (SHARED_CHUNK / 2) as u64,
            limits,
        )
        .unwrap();
    for piece in payload.chunks(SHARED_CHUNK / 2) {
        ingest.push(piece).unwrap();
    }
    ingest.finish().unwrap()
}

pub(crate) fn shared_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = shared_format();
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(512).unwrap())
        .admit(format)
        .unwrap()
}

pub(crate) fn shared_format() -> AdmittedPhysicalRecordFormat {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}