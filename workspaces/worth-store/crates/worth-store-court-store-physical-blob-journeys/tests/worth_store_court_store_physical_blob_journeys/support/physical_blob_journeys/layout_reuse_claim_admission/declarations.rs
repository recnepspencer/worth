#![cfg(all())]

pub(crate) use std::num::NonZeroU64;

pub(crate) use {sha2::Digest, sha2::Sha256};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::RecordAppendDenial,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::BlobChunkReuseClaimV1,
    worth_store_physical_format::BlobChunkReuseClaimV2,
    worth_store_physical_format::BlobGenerationPublicationV1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn persisted(
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap()
}
