pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::CompletedPhysicalCheckpoint,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn unfinished_token(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    horizon: u64,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(horizon).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
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

pub(crate) fn adequate_limits() -> BlobResumeLimits {
    worth_store::physical_runtime::BlobResumeLimits::new(
        std::num::NonZeroU64::new(128).unwrap(),
        std::num::NonZeroU64::new(1024 * 1024).unwrap(),
    )
}

pub(crate) fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub(crate) fn completed_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> CompletedPhysicalCheckpoint {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(10_000)
                .expect("positive deadline"),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        worth_proof::TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        worth_proof::TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        worth_proof::TransitionOutcome::Failed(cause) => {
            panic!("checkpoint start failed: {cause:?}")
        }
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint must complete before expiry assertion: {other:?}"),
    }
}
