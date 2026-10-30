//! Process-kill proof for the V2 manifest-residue selector/Completed gap.

#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_cleanup_crash::*;

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
