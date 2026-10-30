//! Ordinary reopen installs checkpoint custody itself when the selected
//! certificate binds the loaded root and checkpoint source (the clean case).

#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::idempotency_reopen::clean_custody::*;
use {std::fs, std::path::Path};

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store_physical_format::CheckpointSelectiveRecordAggregate;
use {sha2::Digest, sha2::Sha256};
use {
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

use {
    crate::durability_admission::idempotency_reopen::support::checkpoint_records,
    crate::durability_admission::idempotency_reopen::support::prepare,
    crate::durability_admission::idempotency_reopen::support::reseal_record_crc,
    crate::durability_admission::idempotency_reopen::support::success_checkpoint,
};
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

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
        sha2::Sha256::digest(&first)[..],
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
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + 20] ^= 0x01;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    std::fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let policy = durability(&media_owner);
    let (format, _, access) = configuration();
    let worth_proof::TransitionOutcome::Failed(inspection) = media_owner
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, policy,
        ))
        .into_raw()
    else {
        panic!("a certificate body outside its footer aggregate must fail reopen")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::Checkpoint(
                        worth_store::physical_runtime::PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch
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
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + length - NO_RELEASE_ROOT_SHA_FROM_END] ^= 0x01;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    std::fs::write(&path, bytes).unwrap();

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
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, length) = no_release_location(&bytes);
    bytes[offset + length - NO_RELEASE_CHECKPOINT_FROM_END] ^= 0x80;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    reseal_certificate_aggregate(&mut bytes);
    std::fs::write(&path, bytes).unwrap();

    let reopened = serving_from_open(&root);
    assert_custody_unavailable(&reopened, 0x48);
    reopened.close();
}

#[test]
fn tier_certificate_bound_to_another_root_leaves_the_anchor_to_recovery() {
    tampered_tier_requires_c8(|record| record[TIER_ROOT_SHA_OFFSET] ^= 0x01);
}

#[test]
fn tier_completed_frame_beyond_the_wal_cutoff_leaves_the_anchor_to_recovery() {
    tampered_tier_requires_c8(|record| {
        let end = record.len() - TIER_COMPLETED_END_FROM_END;
        record[end..end + 8].copy_from_slice(&(u64::MAX >> 1).to_le_bytes());
    });
}
