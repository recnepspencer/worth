//! Ordinary reopen installs checkpoint custody itself when the selected
//! certificate binds the loaded root and checkpoint source (the clean case).

use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalBindingCompactionReopenFailure, PhysicalCheckpointCaptureFailureKind,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointRequest,
    PhysicalCheckpointStartFailure, PhysicalDurabilityStateReopenFailure,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome, PhysicalRecordOpen,
    PhysicalSignalConstructionFailure, RecordBootstrapDenial, RecordBootstrapFailure,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::CheckpointSelectiveRecordAggregate;

use super::super::super::{
    configuration, durability, media, serving_from_initialization, serving_from_open,
};
use super::support::{checkpoint_records, prepare, reseal_record_crc, success_checkpoint};

/// Prior-marker digest, then prior root digest and sequence, end the marker.
const NO_RELEASE_ROOT_SHA_FROM_END: usize = 4 + 32 + 32 + 8 + 32;
/// The 24-byte checkpoint identity precedes the root generation and digest.
const NO_RELEASE_CHECKPOINT_FROM_END: usize = NO_RELEASE_ROOT_SHA_FROM_END + 8 + 24;

#[test]
fn clean_no_release_reopen_checkpoints_with_the_selected_marker_as_predecessor() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    checkpointed_store(&root);
    let first = no_release_payload(&root);

    let reopened = serving_from_open(&root);
    add_record(&reopened, 0x43);
    success_checkpoint(&reopened, 0x44);
    reopened.close();
    let second = no_release_payload(&root);
    assert_ne!(first, second);
    assert_eq!(
        second[second.len() - 32..],
        Sha256::digest(&first)[..],
        "the reopened ledger is the selected marker, not a fresh genesis"
    );

    let again = serving_from_open(&root);
    add_record(&again, 0x45);
    success_checkpoint(&again, 0x46);
    again.close();
}

#[test]
fn tampered_certificate_body_fails_ordinary_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    checkpointed_store(&root);
    let path = root.join("families/checkpoint.current");
    let mut bytes = fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + 20] ^= 0x01;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let policy = durability(&media_owner);
    let (format, _, access) = configuration();
    let TransitionOutcome::Failed(inspection) = media_owner
        .open_record_store(PhysicalRecordOpen::new(format, access, policy))
        .into_raw()
    else {
        panic!("a certificate body outside its footer aggregate must fail reopen")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::Checkpoint(
                        PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch
                    )
                )
            )
        ),
        "unexpected reopen cause: {cause:?}"
    );
}

#[test]
fn marker_bound_to_another_source_root_leaves_custody_unavailable() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    checkpointed_store(&root);
    let path = root.join("families/checkpoint.current");
    let mut bytes = fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + length - NO_RELEASE_ROOT_SHA_FROM_END] ^= 0x01;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    fs::write(&path, bytes).unwrap();

    let reopened = serving_from_open(&root);
    assert_custody_unavailable(&reopened, 0x47);
    reopened.close();
}

#[test]
fn marker_naming_another_checkpoint_leaves_custody_unavailable() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    checkpointed_store(&root);
    let path = root.join("families/checkpoint.current");
    let mut bytes = fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + length - NO_RELEASE_CHECKPOINT_FROM_END] ^= 0x80;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    fs::write(&path, bytes).unwrap();

    let reopened = serving_from_open(&root);
    assert_custody_unavailable(&reopened, 0x48);
    reopened.close();
}

pub(super) fn assert_custody_unavailable(serving: &ServingPhysicalRuntime, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(TemporalDuration::temporal_duration(10_000).unwrap()),
    );
    let outcome = serving.checkpoints().start(request).into_raw();
    assert!(
        matches!(
            outcome,
            TransitionOutcome::Failed(PhysicalCheckpointStartFailure::Capture(
                PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable
            ))
        ),
        "unverified custody must not checkpoint"
    );
}

fn checkpointed_store(root: &Path) {
    let serving = serving_from_initialization(root);
    add_record(&serving, 0x41);
    success_checkpoint(&serving, 0x42);
    serving.close();
}

fn add_record(serving: &ServingPhysicalRuntime, seed: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([seed; 32]))
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"clean-custody-record");
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}

fn no_release_payload(root: &Path) -> Vec<u8> {
    let bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + 16..offset + length - 4].to_vec()
}

fn no_release_location(bytes: &[u8]) -> (usize, usize) {
    certificate_location(bytes, 7)
}

/// The one certificate record of `kind` (6 tier, 7 release marker).
fn certificate_location(bytes: &[u8], kind: u8) -> (usize, usize) {
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
fn reseal_certificate_aggregate(bytes: &mut [u8]) {
    let mut aggregate = CheckpointSelectiveRecordAggregate::new();
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
const TIER_ROOT_SHA_OFFSET: usize = 16 + 8 + 47 + 24 + 8;
/// The completed-frame witness (start, end, two digests) ends the payload.
const TIER_COMPLETED_END_FROM_END: usize = 4 + 80 - 8;

#[test]
fn tier_certificate_bound_to_another_root_leaves_the_anchor_to_c8() {
    tampered_tier_requires_c8(|record| record[TIER_ROOT_SHA_OFFSET] ^= 0x01);
}

#[test]
fn tier_completed_frame_beyond_the_wal_cutoff_leaves_the_anchor_to_c8() {
    tampered_tier_requires_c8(|record| {
        let end = record.len() - TIER_COMPLETED_END_FROM_END;
        record[end..end + 8].copy_from_slice(&(u64::MAX >> 1).to_le_bytes());
    });
}

/// A resealed tier certificate that no longer binds the loaded root or WAL
/// cutoff cannot certify the anchor, so ordinary reopen denies Serving.
fn tampered_tier_requires_c8(tamper: impl FnOnce(&mut [u8])) {
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
    let mut bytes = fs::read(&path).unwrap();
    let (offset, length) = certificate_location(&bytes, 6);
    tamper(&mut bytes[offset..offset + length]);
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let policy = durability(&media_owner);
    let (format, _, access) = configuration();
    let TransitionOutcome::Denied(denial) = media_owner
        .open_record_store(PhysicalRecordOpen::new(format, access, policy))
        .into_raw()
    else {
        panic!("an unbound tier certificate must leave the anchored root to C8")
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
    );
    denial.into_runtime().close();
}
