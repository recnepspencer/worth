pub(crate) use {std::num::NonZeroU64, std::sync::Barrier, std::thread};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobObjectId,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) const CHUNK_BYTES: u64 = 64 * 1024;

pub(crate) fn declaration(
    object: BlobObjectId,
    scope: &AdmittedBlobScope,
) -> BlobIngestDeclaration {
    worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES).unwrap(),
        2 * CHUNK_BYTES,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap()
}
