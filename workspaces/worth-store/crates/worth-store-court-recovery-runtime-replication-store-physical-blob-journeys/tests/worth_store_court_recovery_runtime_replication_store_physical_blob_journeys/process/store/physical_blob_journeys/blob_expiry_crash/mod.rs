#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_expiry_crash::*;
use {
    std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    std::time::Instant,
};

use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::BlobTerminalDisposition,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::BlobAbandonmentReasonV1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_abort::assert_resume_abandoned,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint,
    crate::physical_blob_journeys::blob_expiry::deadline,
    crate::physical_blob_journeys::blob_expiry::limits,
    crate::physical_blob_journeys::blob_expiry::unfinished,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn expiry_wal_durable_before_root_replacement_replays_one_terminal() {
    assert_killed_expiry("crash-expiry-wal");
}

#[test]
fn expiry_root_replaced_before_completion_reopens_one_terminal() {
    assert_killed_expiry("crash-expiry-root");
}
