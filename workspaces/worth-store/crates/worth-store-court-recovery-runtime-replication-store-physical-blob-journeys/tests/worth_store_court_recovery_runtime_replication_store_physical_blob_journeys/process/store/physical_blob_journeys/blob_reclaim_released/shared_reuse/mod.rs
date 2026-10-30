#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::*;

use {
    crate::physical_blob_journeys::blob_reclaim_released::publish_one,
    crate::physical_blob_journeys::blob_reclaim_released::shared_format,
    crate::physical_blob_journeys::blob_reclaim_released::shared_placement,
};
use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::serving_from_initialization_with_format_and_placement,
};

#[test]
fn retired_tail_slot_does_not_block_selected_successor_reuse() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.retired.tail.successor");
    let payload = multi_chunk_payload();
    let source = publish_one(&serving, &scope, &payload);
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xcf);
    let successor = publish_one(&serving, &scope, &payload);
    assert_selected_reuse_claim(&serving);
    assert_bytes_exact(&serving, &scope, source, &payload);
    assert_bytes_exact(&serving, &scope, successor, &payload);
    serving.close();
}

#[test]
fn released_reuse_child() {
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::fresh_process::run_child();
}

#[test]
fn destination_first_release_drops_claim_then_source_without_stranded_custody() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.release.shared.destination.first");
    let payload = multi_chunk_payload();
    let original = publish_one(&serving, &scope, &payload);
    let original_marker = marker(&serving);
    crate::physical_blob_journeys::blob_expiry::completed_checkpoint(&serving, 0xe1);
    let reused = publish_one(&serving, &scope, &payload);
    let reused_marker = marker(&serving);
    assert_selected_reuse_claim(&serving);
    assert_bytes_exact(&serving, &scope, original, &payload);

    let destination = release(&serving, reused, reused_marker);
    assert_retired(&destination);
    assert_eq!(destination.remaining_payload_records(), 0);
    assert_invisible(&serving, &scope, reused);
    assert!(
        worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records(
            &serving
        )
        .iter()
        .any(|(record, _)| record.allocation_epoch()
            == original_marker.record().allocation_epoch()
            && record.ordinal() == original_marker.record().ordinal()),
        "destination release must preserve the original publication's authoritative route",
    );
    assert_bytes_exact(&serving, &scope, original, &payload);
    let source = release(&serving, original, original_marker);
    assert_retired(&source);
    assert_eq!(source.remaining_payload_records(), 0);
    assert_invisible(&serving, &scope, original);
    serving.close();

    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::fresh_process::assert_open_denied(directory.path());
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::denied_reopen::assert_open_requires_c8_custody(directory.path());
}
