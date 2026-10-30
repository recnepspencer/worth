pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestClaimDenial,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::BlobTerminalDisposition,
    worth_store::physical_runtime::BlobTerminalFailure,
    worth_store::physical_runtime::BlobTerminalLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::admitted_blob_scope_for_replay_boundary,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn unfinished<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 2) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    ingest.push(&[0x5a; CHUNK]).unwrap();
    ingest
}

pub(crate) fn deadline() -> PhysicalMutationDeadline {
    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap()
}

pub(crate) fn limits() -> BlobTerminalLimits {
    worth_store::physical_runtime::BlobTerminalLimits::new(std::num::NonZeroU64::new(128).unwrap())
}

pub(crate) fn resume_limits() -> BlobResumeLimits {
    worth_store::physical_runtime::BlobResumeLimits::new(
        std::num::NonZeroU64::new(128).unwrap(),
        std::num::NonZeroU64::new(1 << 20).unwrap(),
    )
}

pub(crate) fn assert_resume_abandoned(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
) {
    assert!(matches!(
        serving.blobs().unwrap().resume_ingest(
            token,
            scope,
            placement(),
            CHUNK as u64,
            deadline(),
            resume_limits()
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::AlreadyAbandoned)
    ));
}

pub(crate) fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}
