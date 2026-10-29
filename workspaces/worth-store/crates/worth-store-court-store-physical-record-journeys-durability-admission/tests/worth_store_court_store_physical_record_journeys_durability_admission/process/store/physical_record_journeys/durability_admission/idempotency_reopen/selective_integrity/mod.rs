#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::idempotency_reopen::selective_integrity::*;
use {std::fs};

use {sha2::Digest, sha2::Sha256};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalBindingCompactionReopenFailure, worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure, worth_store::physical_runtime::PhysicalIdempotencyReopenFailure, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalSignalConstructionFailure, worth_store::physical_runtime::RecordBootstrapFailure};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {crate::durability_admission::idempotency_reopen::support::checkpoint_records, crate::durability_admission::idempotency_reopen::support::inspect_checkpoint_reopen, crate::durability_admission::idempotency_reopen::support::prepare, crate::durability_admission::idempotency_reopen::support::reseal_record_crc, crate::durability_admission::idempotency_reopen::support::success_checkpoint};

#[test]
fn checksum_valid_binding_mutation_with_stale_footer_aggregate_blocks_ordinary_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    create_checkpoint_with_unsealed_binding(&root, 211);
    let path = root.join("families/checkpoint.current");
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, length) = unsealed_binding_location(&bytes);
    let fingerprint = binding_fingerprint_range(&bytes[offset..offset + length]);
    bytes[offset + fingerprint.start] ^= 0x20;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    std::fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let durability = durability(&media_owner);
    let (format, _, access) = configuration();
    let outcome = media_owner
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability))
        .into_raw();
    let worth_proof::TransitionOutcome::Failed(inspection) = outcome else {
        panic!("stale binding selective aggregate must reject ordinary reopen")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::Idempotency(
                        worth_store::physical_runtime::PhysicalIdempotencyReopenFailure::Checkpoint(
                            worth_store::physical_runtime::PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch
                        )
                    )
                )
            )
        ),
        "unexpected reopen cause: {cause:?}"
    );
}

#[test]
fn damaged_dirty_basis_remains_selectively_skipped_during_ordinary_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    create_checkpoint_with_binding(&root, 212);
    inject_dirty_basis(&root);
    let expected = inspect_checkpoint_reopen(&root);
    assert!(expected.dirty_bytes > 0);
    let path = root.join("families/checkpoint.current");
    let mut bytes = std::fs::read(&path).unwrap();
    let (offset, _) = record_location(&bytes, 2);
    bytes[offset + 16] ^= 0x80;
    std::fs::write(&path, bytes).unwrap();

    let reopened = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    let observation = reopened
        .durability_observation()
        .reopen()
        .expect("dirty-basis bytes are outside ordinary binding-compaction meaning");
    assert_eq!(
        observation.checkpoint_artifact_bytes(),
        expected.artifact_bytes
    );
    assert_eq!(observation.checkpoint_bytes_read(), expected.bytes_read);
    assert_eq!(observation.dirty_body_bytes_skipped(), expected.dirty_bytes);
    assert_eq!(observation.binding_records_read(), expected.binding_records);
    assert_eq!(
        observation.checkpoint_integrity_admissions(),
        expected.binding_records + expected.certificate_records + 3
    );
    reopened.close();
}
