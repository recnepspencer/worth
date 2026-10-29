pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof};
pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobRecordReachability, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalPageSizeClass, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PublishedBlobGeneration, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {crate::physical_blob_journeys::blob_reachability::class_of, crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_reachability::inspection_limits, crate::physical_blob_journeys::blob_reachability::persisted, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records};
pub(crate) use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::serving_from_initialization_with_format_and_placement};

pub(crate) const CHUNK: usize = 256 << 10;

pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, (CHUNK / 2) as u64, read_limits)
        .unwrap();
    for piece in payload.chunks(CHUNK / 2) {
        ingest.push(piece).unwrap();
    }
    ingest.finish().unwrap()
}