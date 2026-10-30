#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_batch_boundary::*;

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::serving_from_initialization_with_placement_and_tail,
    worth_store_testkit_store::fixture::serving_from_open_with_retained_wal_tail,
};

/// Scheduled: 1,025 real C.5 appends and two multiscanned reclamations.
#[test]
#[ignore = "1,025 real 64 KiB chunk records plus bounded reclaim/recovery passes"]
fn abandoned_ingest_reclaims_exact_1024_id_batch_after_restart() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope(SCOPE);
    let placement = batch_placement();
    let serving = serving_from_initialization_with_placement_and_tail(
        directory.path(),
        placement,
        BATCH_RETAINED_WAL_TAIL_BYTES,
    );
    establish_recovery_frontier(&serving);
    let token = abandoned_1025_chunk_ingest(&serving, &scope, placement);
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    let selected = selected_chunk_identities(&serving, token);
    assert_eq!(selected.len(), CHUNKS, "all claims survived fresh recovery");
    let original_root = root_generation(&serving);
    let append_attempts = serving.media_counters().append_attempts();
    let denied =
        serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope, placement, INSPECTION_BYTE_DENIAL));
    assert!(matches!(
        denied,
        Err(worth_store::physical_runtime::BlobReclaimFailure::InspectedByteBoundExhausted)
    ));
    assert_eq!(root_generation(&serving), original_root);
    assert_eq!(serving.media_counters().append_attempts(), append_attempts);

    let mut first = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    finish_durable_batch(&serving, "frontier-and-last-chunk", &mut first);
    assert_durable_batch("frontier-and-last-chunk", &first);
    assert_eq!(first.remaining_payload_records(), 1024);
    assert_eq!(first.dropped_records().len(), 17);
    assert!(first.dropped_records().contains(&selected[1024]));
    assert_eq!(selected_chunk_identities(&serving, token), selected[..1024]);
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    assert_eq!(selected_chunk_identities(&serving, token), selected[..1024]);
    let mut second = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    finish_durable_batch(&serving, "1024-chunk-drop", &mut second);
    assert_durable_batch("1024-chunk-drop", &second);
    assert_eq!(second.dropped_records().len(), 1024);
    assert_eq!(second.remaining_payload_records(), 0);
    let mut expected_drop_ids = selected[..1024].to_vec();
    expected_drop_ids.sort_unstable();
    assert_eq!(second.dropped_records(), expected_drop_ids.as_slice());
    assert!(selected_chunk_identities(&serving, token).is_empty());
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    let root_before_repeat = root_generation(&serving);
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.bytes_released(), 0);
    assert_eq!(root_generation(&serving), root_before_repeat);
    serving.close();
}
