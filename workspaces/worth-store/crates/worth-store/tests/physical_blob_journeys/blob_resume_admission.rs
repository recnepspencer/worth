use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    BlobResumeFailure, BlobResumeLimits, BlobResumeToken, CompletedPhysicalCheckpoint,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::{
    blob_frontier::selected_blob_records,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

const CHUNK: usize = 64 * 1024;

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
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            adequate_limits(),
        ),
        Err(BlobResumeFailure::Expired)
    ));
    assert_eq!(selected_root(&serving), root_before);
    assert_eq!(selected_blob_records(&serving), records_before);
    drop(blobs);
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
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobResumeLimits::new(
                NonZeroU64::new(1).unwrap(),
                NonZeroU64::new(1024 * 1024).unwrap(),
            ),
        ),
        Err(BlobResumeFailure::ScanBoundExhausted)
    ));
    assert_eq!(selected_root(&serving), root_before);
    assert_eq!(selected_blob_records(&serving), records_before);

    let resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            adequate_limits(),
        )
        .expect("exhausted inspection must release its live session claim");
    assert_eq!(resumed.frontier().bytes(), CHUNK as u64);
    assert_eq!(resumed.frontier().next_chunk_ordinal(), 1);
    assert_eq!(selected_root(&serving), root_before);
    drop(resumed);
    drop(blobs);
    serving.close();
}

fn unfinished_token(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    horizon: u64,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(horizon).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    let token = ingest.resume_token();
    ingest.push(&[0x5a; CHUNK]).unwrap();
    assert_eq!(ingest.frontier().bytes(), CHUNK as u64);
    drop(ingest);
    token
}

fn adequate_limits() -> BlobResumeLimits {
    BlobResumeLimits::new(
        NonZeroU64::new(128).unwrap(),
        NonZeroU64::new(1024 * 1024).unwrap(),
    )
}

fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

fn completed_checkpoint(serving: &ServingPhysicalRuntime, key: u8) -> CompletedPhysicalCheckpoint {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(10_000).expect("positive deadline"),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        TransitionOutcome::Failed(cause) => panic!("checkpoint start failed: {cause:?}"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint must complete before expiry assertion: {other:?}"),
    }
}
