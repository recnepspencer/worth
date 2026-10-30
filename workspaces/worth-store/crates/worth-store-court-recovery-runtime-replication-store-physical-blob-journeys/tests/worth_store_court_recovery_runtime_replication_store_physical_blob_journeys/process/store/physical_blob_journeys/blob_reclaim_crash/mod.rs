#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_crash::*;
use {
    std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    std::time::Instant,
};

use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobResumeToken,
};
use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_initialization_with_wal_segment_bytes,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::fixture::serving_from_open_with_wal_segment_bytes,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn selected_manifest_kill_retains_failed_ingest_payload_without_a_drop() {
    assert_killed_reclaim("crash-reclaim-manifest", false);
}

#[test]
fn selected_reservation_kill_without_descriptor_does_not_strand_payload() {
    assert_killed_reclaim("crash-reclaim-reservation", false);
}

#[test]
fn selected_reservation_still_cleans_after_lease_expiry_and_wal_pruning() {
    assert_killed_reclaim(AGED_RESERVATION, false);
}

#[test]
fn durable_descriptor_wal_kill_replays_exact_failed_ingest_drop() {
    assert_killed_reclaim("crash-reclaim-wal", true);
}
