pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalBlobFacade,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK_BYTES: usize = 64 * 1024;

pub(crate) fn begin<'runtime>(
    blobs: &PhysicalBlobFacade<'runtime>,
    scope: &AdmittedBlobScope,
) -> BlobIngestSession<'runtime> {
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        blobs.issue_object_id(limits).unwrap(),
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap();
    blobs
        .begin_ingest(declaration, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap()
}

pub(crate) fn selected_publication(serving: &ServingPhysicalRuntime) -> PersistedRecordIdentity {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation selected")
        .record()
}

pub(crate) fn reuse_claims(serving: &ServingPhysicalRuntime) -> Vec<PersistedRecordIdentity> {
    selected_blob_records(serving)
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(value)) => {
                Some(value.claim().source_publication())
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn read_exact(
    blobs: &PhysicalBlobFacade<'_>,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
    payload: &[u8],
) -> u64 {
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(1).unwrap());
    let mut read = blobs
        .read(published, scope, 0, payload.len() as u64, limits)
        .unwrap();
    let mut result = Vec::new();
    let mut transfer = [0_u8; 4096];
    while read.remaining_bytes() != 0 {
        let count = read.read_next(&mut transfer).unwrap();
        result.extend_from_slice(&transfer[..count]);
    }
    assert_eq!(result, payload);
    read.observation().reuse_source_selected_reads()
}
