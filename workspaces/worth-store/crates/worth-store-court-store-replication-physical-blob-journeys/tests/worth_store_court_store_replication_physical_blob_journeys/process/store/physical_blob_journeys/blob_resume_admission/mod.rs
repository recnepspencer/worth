#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume_admission::*;

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn completed_checkpoint_crossing_original_horizon_denies_unchanged_token_without_blob_write() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.resume.expiry.scope");
    let token = unfinished_token(&serving, &scope, 1);
    let wire = token.encode();
    let declared_max = u64::from_le_bytes(wire[108..116].try_into().unwrap());

    let mut completed_sequence = 0;
    for key in 1..=3 {
        completed_sequence = completed_checkpoint(&serving, key)
            .footer()
            .identity()
            .sequence()
            .get();
        if completed_sequence > declared_max {
            break;
        }
    }
    assert!(
        completed_sequence > declared_max,
        "real completed checkpoint must cross the original declaration horizon"
    );
    let root_before = selected_root(&serving);
    let records_before = selected_blob_records(&serving);

    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            adequate_limits(),
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::Expired)
    ));
    assert_eq!(selected_root(&serving), root_before);
    assert_eq!(selected_blob_records(&serving), records_before);
    serving.close();
}

#[test]
fn exact_scan_bound_denies_without_write_and_releases_session_claim() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.resume.scan-bound.scope");
    let token = unfinished_token(&serving, &scope, 16);
    let root_before = selected_root(&serving);
    let records_before = selected_blob_records(&serving);
    assert!(
        records_before.len() > 1,
        "one admitted scan row cannot establish the selected declaration and chunk"
    );

    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            worth_store::physical_runtime::BlobResumeLimits::new(
                std::num::NonZeroU64::new(1).unwrap(),
                std::num::NonZeroU64::new(1024 * 1024).unwrap(),
            ),
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::ScanBoundExhausted)
    ));
    assert_eq!(selected_root(&serving), root_before);
    assert_eq!(selected_blob_records(&serving), records_before);

    let resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            adequate_limits(),
        )
        .expect("exhausted inspection must release its live session claim");
    assert_eq!(resumed.frontier().bytes(), CHUNK as u64);
    assert_eq!(resumed.frontier().next_chunk_ordinal(), 1);
    assert_eq!(selected_root(&serving), root_before);
    drop(resumed);
    serving.close();
}
