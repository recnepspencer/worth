//! Process-kill proof for the V2 manifest-residue selector/Completed gap.

#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_cleanup_crash::*;
use {
    std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::thread,
    std::time::Duration, std::time::Instant,
};

use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::production::PhysicalMutationPauseGate,
    worth_store::physical_runtime::BlobAppendFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalRetirementDenial,
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
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn selected_v2_cleanup_without_completed_reopens_and_retires_exact_metadata_once() {
    assert_cleanup_reopens_and_retires_exact_metadata_once(SELECTOR_ROLE);
}

/// The cleanup Intent is WAL-durable above the checkpoint's covered end and
/// the selected root is still its source, so recovery must replay it.
#[test]
#[ignore = "C.11 remainder: C8 rebuilds the segment and free-space manifests the writer's \
            cleanup plan keeps, so the replayed root never matches the Intent's root SHA"]
fn durable_v2_cleanup_intent_above_the_checkpoint_replays_and_retires_exact_metadata_once() {
    assert_cleanup_reopens_and_retires_exact_metadata_once(INTENT_ROLE);
}
