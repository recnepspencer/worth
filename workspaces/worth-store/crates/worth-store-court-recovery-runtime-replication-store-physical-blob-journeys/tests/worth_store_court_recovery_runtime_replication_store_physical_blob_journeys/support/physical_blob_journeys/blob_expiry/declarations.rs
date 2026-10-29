pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::production::PhysicalCheckpointStep, worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestClaimDenial, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobTerminalDisposition, worth_store::physical_runtime::BlobTerminalFailure, worth_store::physical_runtime::BlobTerminalLimits, worth_store::physical_runtime::CompletedPhysicalCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointHandle, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalWalReclamationObservation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobAbandonmentReasonV1, worth_store_physical_format::BlobRecordV1};



pub(crate) use {crate::physical_blob_journeys::blob_abort::assert_resume_abandoned, crate::blob_crash::recovery::recover_closed_store, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization_with_wal_segment_bytes, worth_store_testkit_store::fixture::serving_from_open_with_wal_segment_bytes};

pub(crate) const CHUNK: usize = 64 * 1024;
pub(crate) const WAL_SEGMENT_BYTES: u64 = 256 * 1024;

pub(crate) fn assert_not_expired(
    serving: &ServingPhysicalRuntime,
    token: worth_store::physical_runtime::BlobResumeToken,
    scope: &AdmittedBlobScope,
    selected: u64,
    maximum: u64,
) {
    let before = selected_blob_records(serving);
    let media_before = serving.media_counters();
    assert!(matches!(
        serving
            .blobs()
            .unwrap()
            .expire_ingest(token, scope, placement(), deadline(), limits()),
        Err(worth_store::physical_runtime::BlobTerminalFailure::NotExpired {
            selected_checkpoint_sequence,
            maximum_checkpoint_sequence,
        }) if selected_checkpoint_sequence == selected && maximum_checkpoint_sequence == maximum
    ));
    assert_eq!(selected_blob_records(serving), before);
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after.positioned_write_attempts(),
        media_before.positioned_write_attempts()
    );
}

pub(crate) fn unfinished<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> worth_store::physical_runtime::BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 2) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(1).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
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

pub(crate) fn completed_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> CompletedPhysicalCheckpoint {
    completed(start_checkpoint(serving, key))
}

pub(crate) fn start_checkpoint(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointHandle {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(10_000).expect("positive deadline"),
        ),
    );
    match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        worth_proof::TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        worth_proof::TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        worth_proof::TransitionOutcome::Failed(cause) => panic!("checkpoint start failed: {cause:?}"),
    }
}

pub(crate) fn completed(handle: PhysicalCheckpointHandle) -> CompletedPhysicalCheckpoint {
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint must complete before expiry assertion: {other:?}"),
    }
}

pub(crate) fn append_padding(serving: &ServingPhysicalRuntime, ordinal: u8) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
            [0xe0 + ordinal; 32],
        ))
        .unwrap();
    let frame = [ordinal; CHUNK];
    let batch = worth_store::physical_runtime::RecordAppendBatch::try_from_iter([frame.as_slice()]).unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                batch,
                placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("WAL-rotation padding must prepare through ordinary C5");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}

pub(crate) fn selected_blob_frames(
    serving: &ServingPhysicalRuntime,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    selected_blob_records(serving)
        .into_iter()
        .filter(|(_, bytes)| bytes.starts_with(b"WRC11BLB"))
        .collect()
}

pub(crate) fn assert_checkpoint_wal_range(
    root: &Path,
    cutoff: u64,
    stage: &str,
) -> crate::physical_blob_journeys::blob_expiry::wal_oracle::IndependentWalInventory {
    let inventory = crate::physical_blob_journeys::blob_expiry::wal_oracle::inspect_wal_inventory(root)
        .unwrap_or_else(|denial| panic!("{stage}: retained WAL inventory denied: {denial:?}"));
    let (first, active_end) = inventory
        .lsn_range()
        .unwrap_or_else(|| panic!("{stage}: retained WAL inventory is empty"));
    assert!(
        first <= cutoff && cutoff <= active_end,
        "{stage}: checkpoint cutoff {cutoff} outside retained WAL [{first}, {active_end}]"
    );
    inventory
}

pub(crate) fn wal_segment_path(root: &Path, (segment, generation): (u64, u64)) -> std::path::PathBuf {
    root.join("families")
        .join("wal")
        .join(format!("segment-{segment}-generation-{generation}.wal"))
}