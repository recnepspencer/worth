#![cfg(all())]

pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobDedupeFailure,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDeferral,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::PhysicalLayoutDenial,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobChunkFrameV1,
    worth_store_physical_format::BlobChunkOccurrenceV1, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::DecodedBlobChunkFrameV1,
};

pub(crate) use {
    crate::layout_dedupe_collision::selected_evidence::persisted,
    crate::layout_dedupe_collision::selected_evidence::selected_chunk,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_abort::limits as terminal_limits,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    crate::physical_blob_journeys::blob_reclaim::request as reclaim_request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
