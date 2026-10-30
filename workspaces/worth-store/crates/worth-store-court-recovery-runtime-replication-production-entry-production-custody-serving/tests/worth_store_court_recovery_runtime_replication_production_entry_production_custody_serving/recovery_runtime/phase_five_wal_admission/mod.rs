use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};

#[test]
fn interrupted_terminal_first_frame_preserves_the_complete_prior_segment() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (first_path, first) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "complete-prior-frame",
            b"complete",
        );
    let (second_path, second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            2,
            3,
            4,
            "interrupted-newest-frame",
            b"interrupted",
        );
    std::fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    std::fs::write(first_path, &first).unwrap();
    std::fs::write(second_path, &second[..37]).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().valid_wal_frames, 1);
    assert_eq!(discovered.counters().valid_wal_bytes, first.len() as u64);
    assert_eq!(discovered.counters().torn_suffix_frames, 1);
    assert_eq!(discovered.counters().torn_suffix_bytes, 37);
    assert_eq!(discovered.counters().interrupted_wal_start_residue, 1);
    assert_eq!(discovered.counters().wal_corruption_denials, 0);
    assert_eq!(discovered.counters().wal_integrity_attempts, 2);
    assert_eq!(discovered.counters().wal_integrity_admissions, 1);
    assert_eq!(discovered.counters().wal_integrity_rejections, 1);
    assert_eq!(discovered.counters().wal_owner_projections, 1);
    assert_eq!(discovered.counters().wal_owner_decoder_entries, 0);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.wal_segment_count(), 1);
    assert_eq!(selected.wal_frame_count(), 1);
    assert_eq!(selected.residue_count(), 1);
    assert_eq!(selected.wal_integrity_observations().len(), 2);
    assert_eq!(
        selected.wal_integrity_observations()[0].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
    );
    assert!(matches!(
        selected.wal_integrity_observations()[1].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
    ));
    let _ = selected.cancel_before_reconstruction();
}

#[test]
fn corrupt_second_frame_is_rejected_before_any_wal_decoder_entry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (path, first) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "first-exact-frame",
            b"first",
        );
    let (_, second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            3,
            4,
            "corrupt-second-frame",
            b"second",
        );
    let mut bytes = first;
    bytes.extend_from_slice(&second);
    *bytes.last_mut().unwrap() ^= 0x5a;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.wal_integrity_attempts, 2);
    assert_eq!(counters.wal_integrity_admissions, 1);
    assert_eq!(counters.wal_integrity_rejections, 1);
    assert_eq!(counters.wal_owner_projections, 1);
    assert_eq!(counters.wal_owner_decoder_entries, 0);
    assert_eq!(counters.wal_corruption_denials, 1);
    let blocked = expect_blocked(discovered.select().err().unwrap());
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::WalInventory)
    );
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::WalIntegrity(_)
        )));
}

#[test]
fn truncated_nonterminal_start_is_corruption_even_when_a_later_segment_is_valid() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (first_path, first) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "truncated-nonterminal",
            b"first",
        );
    let (second_path, second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            2,
            3,
            4,
            "valid-terminal",
            b"second",
        );
    std::fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    std::fs::write(first_path, &first[..37]).unwrap();
    std::fs::write(second_path, second).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.wal_corruption_denials, 1);
    assert_eq!(counters.interrupted_wal_start_residue, 0);
    assert_eq!(counters.torn_suffix_frames, 0);
    assert_eq!(counters.wal_integrity_attempts, 2);
    assert_eq!(counters.wal_integrity_admissions, 1);
    assert_eq!(counters.wal_integrity_rejections, 1);
    assert_eq!(counters.wal_owner_projections, 1);
    assert_eq!(counters.wal_owner_decoder_entries, 0);
    let blocked = expect_blocked(discovered.select().err().unwrap());
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::WalInventory)
    );
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
}

#[test]
fn empty_terminal_segment_keeps_recovery_and_integrity_counters_distinct() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    // C.11 continuation: segment 1 holds the cutoff's covered WAL anchor, so
    // the empty terminal residue is the next segment.
    publish_synthetic_covered_wal(&root);
    let path = root
        .join("families")
        .join("wal")
        .join("segment-2-generation-1.wal");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, []).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    // The covered anchor is the only frame: one C.9 admission and projection.
    // The empty terminal segment is C.8 residue and one C.9 rejection.
    assert_eq!(counters.wal_frames, 1);
    assert_eq!(counters.trailing_empty_wal_residue, 1);
    assert_eq!(counters.torn_suffix_frames, 0);
    assert_eq!(counters.wal_corruption_denials, 0);
    assert_eq!(counters.wal_integrity_attempts, 2);
    assert_eq!(counters.wal_integrity_admissions, 1);
    assert_eq!(counters.wal_integrity_rejections, 1);
    assert_eq!(counters.wal_owner_projections, 1);
    assert_eq!(counters.wal_owner_decoder_entries, 0);
    let selected = discovered.select().unwrap();
    let observations = selected.wal_integrity_observations();
    assert_eq!(observations.len(), 2);
    assert!(matches!(
        observations[0].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
    ));
    assert!(matches!(
        observations[1].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
    ));
}

#[test]
fn wal_frame_budget_block_preserves_completed_admission_evidence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (path, first) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "budget-first",
            b"first",
        );
    let (_, second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            3,
            4,
            "budget-second",
            b"second",
        );
    let mut bytes = first;
    bytes.extend_from_slice(&second);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.wal_frames = 1;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();

    let blocked = expect_blocked(
        admitted_recovery_with_limits(&root, limits)
            .discover()
            .err()
            .unwrap(),
    );
    let counters = blocked.evidence().counters;
    assert_eq!(counters.wal_integrity_attempts, 1);
    assert_eq!(counters.wal_integrity_admissions, 1);
    assert_eq!(counters.wal_owner_projections, 1);
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 1);
    let limit = blocked.cause().limit().unwrap();
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::WalFrames
    );
    assert_eq!((limit.observed(), limit.admitted()), (2, 1));
}

/// The second segment's reader is handed what the first left of the frames
/// recovery admits: its refusal is stated in recovery's counts.
#[test]
fn a_later_segment_past_the_frames_left_states_recovery_frames() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let frame = |segment, lsn, name: &str, payload: &[u8]| {
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            segment,
            lsn,
            lsn + 1,
            name,
            payload,
        )
    };
    let (first_path, first) = frame(1, 2, "segments-first", b"first");
    let (second_path, mut second) = frame(2, 3, "segments-second", b"second");
    second.extend_from_slice(&frame(2, 4, "segments-third", b"third").1);
    std::fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    std::fs::write(first_path, first).unwrap();
    std::fs::write(second_path, second).unwrap();
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.wal_frames = 2;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();

    let blocked = expect_blocked(
        admitted_recovery_with_limits(&root, limits)
            .discover()
            .err()
            .unwrap(),
    );
    // Handed the one frame left, the second segment held two beside the
    // first segment's one.
    let limit = blocked.cause().limit().expect("the frames recovery admits");
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::WalFrames
    );
    assert_eq!((limit.observed(), limit.admitted()), (3, 2));
}

#[test]
fn hostile_second_frame_length_is_rejected_before_owner_decode() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (path, first) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "hostile-first",
            b"first",
        );
    let (_, mut second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            3,
            4,
            "hostile-second",
            b"second",
        );
    second[44..52].copy_from_slice(&(u64::MAX - 200).to_le_bytes());
    let mut bytes = first;
    bytes.extend_from_slice(&second);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.wal_integrity_attempts, 2);
    assert_eq!(counters.wal_integrity_admissions, 1);
    assert_eq!(counters.wal_integrity_rejections, 1);
    assert_eq!(counters.wal_owner_projections, 1);
    assert_eq!(counters.wal_owner_decoder_entries, 0);
    assert_eq!(counters.wal_corruption_denials, 1);
}

#[path = "denial_observations/mod.rs"]
pub(crate) mod denial_observations;
