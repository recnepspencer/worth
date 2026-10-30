#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume_crash::*;
use {std::fs, std::num::NonZeroU64, std::time::Duration};

use {
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn killed_chunk_writer_resumes_selected_prefix_without_duplicate_claim() {
    resume_killed_prefix("crash-chunk");
}

#[test]
fn killed_frontier_writer_discards_partial_input_and_resumes_exact_selected_prefix() {
    resume_killed_prefix("crash-frontier-partial");
}
