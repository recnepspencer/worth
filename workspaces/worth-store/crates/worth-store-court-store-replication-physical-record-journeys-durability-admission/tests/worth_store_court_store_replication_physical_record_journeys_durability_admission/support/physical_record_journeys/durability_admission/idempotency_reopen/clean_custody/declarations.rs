//! Ordinary reopen installs checkpoint custody itself when the selected
//! certificate binds the loaded root and checkpoint source (the clean case).

pub(crate) use {std::fs, std::path::Path};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use worth_store_physical_format::CheckpointSelectiveRecordAggregate;
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::PhysicalBindingCompactionReopenFailure,
    worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalCheckpointStartFailure,
    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalSignalConstructionFailure,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordBootstrapFailure,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    crate::durability_admission::idempotency_reopen::support::checkpoint_records,
    crate::durability_admission::idempotency_reopen::support::prepare,
    crate::durability_admission::idempotency_reopen::support::reseal_record_crc,
    crate::durability_admission::idempotency_reopen::support::success_checkpoint,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

/// Prior-marker digest, then prior root digest and sequence, end the marker.
pub(crate) const NO_RELEASE_ROOT_SHA_FROM_END: usize = 4 + 32 + 32 + 8 + 32;
/// The 24-byte checkpoint identity precedes the root generation and digest.
pub(crate) const NO_RELEASE_CHECKPOINT_FROM_END: usize = NO_RELEASE_ROOT_SHA_FROM_END + 8 + 24;

pub(crate) fn assert_custody_unavailable(serving: &ServingPhysicalRuntime, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(10_000).unwrap(),
        ),
    );
    let outcome = serving.checkpoints().start(request).into_raw();
    assert!(
        matches!(
            outcome,
            worth_proof::TransitionOutcome::Failed(worth_store::physical_runtime::PhysicalCheckpointStartFailure::Capture(
                worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable
            ))
        ),
        "unverified custody must not checkpoint"
    );
}

pub(crate) fn checkpointed_store(root: &Path) {
    let serving = serving_from_initialization(root);
    add_record(&serving, 0x41);
    success_checkpoint(&serving, 0x42);
    serving.close();
}

pub(crate) fn add_record(serving: &ServingPhysicalRuntime, seed: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([seed; 32]),
        )
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"clean-custody-record");
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}

pub(crate) fn no_release_payload(root: &Path) -> Vec<u8> {
    let bytes = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + 16..offset + length - 4].to_vec()
}

pub(crate) fn no_release_location(bytes: &[u8]) -> (usize, usize) {
    certificate_location(bytes, 7)
}

/// The one certificate record of `kind` (6 tier, 7 release marker).
pub(crate) fn certificate_location(bytes: &[u8], kind: u8) -> (usize, usize) {
    let mut found = checkpoint_records(bytes)
        .into_iter()
        .filter(|record| record[9] == kind);
    let target = found.next().expect("checkpoint carries the certificate");
    assert!(found.next().is_none());
    (
        target.as_ptr() as usize - bytes.as_ptr() as usize,
        target.len(),
    )
}

/// Recomputes the certified footer's certificate aggregate and footer CRC.
pub(crate) fn reseal_certificate_aggregate(bytes: &mut [u8]) {
    let mut aggregate = worth_store_physical_format::CheckpointSelectiveRecordAggregate::new();
    for record in checkpoint_records(bytes) {
        if matches!(record[9], 6 | 7) {
            aggregate.include(record).unwrap();
        }
    }
    let summary = aggregate.summary();
    let footer = checkpoint_records(bytes).last().unwrap().len();
    let start = bytes.len() - footer;
    let footer = &mut bytes[start..];
    footer[152..160].copy_from_slice(&summary.record_count().to_le_bytes());
    footer[160..168].copy_from_slice(&summary.encoded_bytes().to_le_bytes());
    footer[168..200].copy_from_slice(&summary.digest());
    reseal_record_crc(footer);
}

/// The tier payload's source-root digest follows its length-prefixed
/// 47-byte domain, the checkpoint identity and the root generation.
pub(crate) const TIER_ROOT_SHA_OFFSET: usize = 16 + 8 + 47 + 24 + 8;
/// The completed-frame witness (start, end, two digests) ends the payload.
pub(crate) const TIER_COMPLETED_END_FROM_END: usize = 4 + 80 - 8;

/// A resealed tier certificate that no longer binds the loaded root or WAL
/// cutoff cannot certify the anchor, so ordinary reopen denies Serving.
pub(crate) fn tampered_tier_requires_c8(tamper: impl FnOnce(&mut [u8])) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    serving
        .certification_activate_tier_epoch(placement)
        .unwrap();
    // A later root keeps the digest apart from the activation candidate's.
    add_record(&serving, 0x49);
    success_checkpoint(&serving, 0x4a);
    serving.close();
    let path = root.join("families/checkpoint.current");
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, length) = certificate_location(&bytes, 6);
    tamper(&mut bytes[offset..offset + length]);
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    std::fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let policy = durability(&media_owner);
    let (format, _, access) = configuration();
    let worth_proof::TransitionOutcome::Denied(denial) = media_owner
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, policy,
        ))
        .into_raw()
    else {
        panic!("an unbound tier certificate must leave the anchored root to C8")
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
    );
    denial.into_runtime().close();
}
