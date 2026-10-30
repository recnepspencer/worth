#[allow(unused_imports)]
use crate::phase_five_checkpoint_cutover::*;

use {
    worth_store_physical_format::CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};

#[test]
fn production_discovery_projects_all_five_checkpoint_families_without_raw_decode() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let (checkpoint, bytes) = checkpoint_with_dirty_and_bindings(store, 1);
    write_checkpoint(&root, &bytes);
    // C.11 continuation: the cutoff needs its retained covered WAL anchor.
    publish_synthetic_covered_wal(&root);

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
    // Seven checkpoint records plus the source-root manifest they bind.
    assert_eq!(observations.len(), 8);
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::CheckpointIntegrity(
                worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::Integrity(
                    _
                )
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
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();

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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::CheckpointIntegrity(
                worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::BindingRecordLimit
            )
        )));
    let limit = blocked
        .cause()
        .limit()
        .expect("the binding count is a limit");
    assert_eq!(
        (
            blocked.cause().phase(),
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        ),
        (
            worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Checkpoint,
            worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::OperationBindings,
            2,
            1
        )
    );
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
        let limits =
            worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
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
                worth_store_recovery_runtime::PhysicalRecoverySourceDenial::CheckpointIntegrity(
                    worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::ScopeMismatch
                )
            )));
    }
}
