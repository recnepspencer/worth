mod phase_three_support;

use std::num::NonZeroU64;

use phase_three_support::*;
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource, RecordArtifactFile, RecordFrameCoordinate,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryCheckpointIntegrityDenial, PhysicalRecoveryLimits, PhysicalRecoverySourceDenial,
};

#[test]
fn production_discovery_projects_all_five_checkpoint_families_without_raw_decode() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let (checkpoint, bytes) = checkpoint_with_dirty_and_bindings(store, 1);
    write_checkpoint(&root, &bytes);

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().checkpoint_candidates, 1);
    assert_eq!(discovered.counters().checkpoint_integrity_attempts, 7);
    assert_eq!(discovered.counters().checkpoint_integrity_admissions, 7);
    assert_eq!(discovered.counters().checkpoint_integrity_rejections, 0);
    assert_eq!(discovered.counters().checkpoint_owner_projections, 5);
    assert_eq!(discovered.counters().checkpoint_owner_decoder_entries, 0);

    let selected = discovered.select().unwrap();
    assert_eq!(selected.checkpoint_identity(), Some(checkpoint));
    assert_eq!(selected.compaction_generation(), Some(1));
    assert_eq!(selected.root_generation(), 1);
    let observations = selected.integrity_observations().to_vec();
    assert_eq!(observations.len(), 7);
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(refusal) =
        selected.cancel_before_reconstruction()
    else {
        panic!("expected cancellation");
    };
    assert_eq!(refusal.integrity_observations(), observations);
}

#[test]
fn checksum_valid_binding_mutation_is_rejected_by_selective_aggregate_before_projection() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let (_, mut bytes) = checkpoint_with_dirty_and_bindings(store, 1);
    let binding_offset = CHECKPOINT_STREAM_HEADER_RECORD_BYTES
        + CHECKPOINT_DIRTY_FRAME_RECORD_BYTES
        + CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES;
    let payload_offset = binding_offset + 16;
    bytes[payload_offset] ^= 0x40;
    reseal_record_crc(&mut bytes[binding_offset..binding_offset + 27]);
    write_checkpoint(&root, &bytes);

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().checkpoint_integrity_attempts, 6);
    assert_eq!(discovered.counters().checkpoint_integrity_admissions, 5);
    assert_eq!(discovered.counters().checkpoint_integrity_rejections, 1);
    assert_eq!(discovered.counters().checkpoint_owner_projections, 0);
    assert_eq!(discovered.counters().checkpoint_owner_decoder_entries, 0);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("selective aggregate mismatch must block"),
    );
    let observations = blocked.evidence().integrity_observations();
    assert_eq!(observations.len(), 6);
    assert_eq!(observations.last().unwrap().scope().artifact_family(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter);
    assert!(matches!(
        observations.last().unwrap().outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(_)
    ));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::CheckpointIntegrity(
                PhysicalRecoveryCheckpointIntegrityDenial::Integrity(_)
            )
        )));
}

#[test]
fn checkpoint_binding_record_limit_is_a_typed_discovery_denial() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let (_, bytes) = checkpoint_with_dirty_and_bindings(store, 2);
    write_checkpoint(&root, &bytes);
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.operation_bindings = 1;
    let limits = PhysicalRecoveryLimits::admit(declaration).unwrap();

    let discovered = admitted_recovery_with_limits(&root, limits)
        .discover()
        .unwrap();
    assert_eq!(discovered.counters().checkpoint_integrity_attempts, 2);
    assert_eq!(discovered.counters().checkpoint_integrity_admissions, 2);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("binding record limit must block"),
    );
    assert_eq!(blocked.evidence().integrity_observations().len(), 2);
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::CheckpointIntegrity(
                PhysicalRecoveryCheckpointIntegrityDenial::BindingRecordLimit {
                    observed: 2,
                    admitted: 1
                }
            )
        )));
}

#[test]
fn tiny_checkpoint_rejects_resealed_huge_counts_before_body_admission() {
    // Footer record payload offsets are pinned by the persisted format, not
    // obtained from the production parser under test (16-byte record prefix).
    for count_offset in [16 + 24, 16 + 88] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let store = initialize_store(&root);
        publish_synthetic_genesis(&root, store);
        let (_, mut bytes) = checkpoint_with_dirty_and_bindings(store, 1);
        let footer_offset =
            bytes.len() - worth_store_physical_format::CHECKPOINT_STREAM_FOOTER_RECORD_BYTES;
        let footer = &mut bytes[footer_offset..];
        footer[count_offset..count_offset + 8].copy_from_slice(&16_000_000_u64.to_le_bytes());
        reseal_record_crc(footer);
        write_checkpoint(&root, &bytes);
        let mut declaration = limit_declaration(2, 8, 8 * 1024);
        declaration.operation_bindings = 16_000_000;
        declaration.dirty_frames = 16_000_000;
        let limits = PhysicalRecoveryLimits::admit(declaration).unwrap();
        let discovered = admitted_recovery_with_limits(&root, limits)
            .discover()
            .unwrap();
        assert_eq!(discovered.counters().checkpoint_integrity_attempts, 3);
        assert_eq!(discovered.counters().checkpoint_integrity_admissions, 2);
        assert_eq!(discovered.counters().checkpoint_owner_projections, 0);
        let blocked = expect_blocked(
            discovered
                .select()
                .err()
                .expect("impossible count must block"),
        );
        let observations = blocked.evidence().integrity_observations();
        assert_eq!(observations.len(), 3);
        assert!(matches!(
            observations.last().unwrap().outcome(),
            worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(_)
        ));
        assert!(blocked
            .evidence()
            .source_denials
            .iter()
            .any(|denial| matches!(
                denial,
                PhysicalRecoverySourceDenial::CheckpointIntegrity(
                    PhysicalRecoveryCheckpointIntegrityDenial::ScopeMismatch
                )
            )));
    }
}

fn checkpoint_with_dirty_and_bindings(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    binding_count: usize,
) -> (PhysicalCheckpointIdentity, Vec<u8>) {
    let checkpoint = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let source = PhysicalCheckpointSource::concurrent(
        checkpoint,
        CheckpointWalSourceRange::new(1, 2).unwrap(),
        CheckpointRootBasis::new(1, 7),
        1,
    );
    let (mut encoder, header) = CheckpointStreamEncoder::begin(source);
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 1 }, 0, 1)
            .unwrap();
    let dirty = encoder.encode_dirty_basis(CheckpointDirtyFrameBasis::new(coordinate, 1));
    let (mut compaction, compaction_record) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 2).unwrap());
    let mut bindings = Vec::new();
    for index in 0..binding_count {
        bindings.push(
            compaction
                .encode_binding_record(&[b'b', b'i', b'n', b'd', index as u8, 1, 2])
                .unwrap(),
        );
    }
    let (_, footer) = compaction.finish();
    let mut bytes = header;
    bytes.extend_from_slice(&dirty);
    bytes.extend_from_slice(&compaction_record);
    for binding in bindings {
        bytes.extend_from_slice(&binding);
    }
    bytes.extend_from_slice(&footer);
    (checkpoint, bytes)
}

fn write_checkpoint(root: &std::path::Path, bytes: &[u8]) {
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
}

fn reseal_record_crc(record: &mut [u8]) {
    let checksum_offset = record.len() - 4;
    let checksum = crc32c(&record[..checksum_offset]);
    record[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
