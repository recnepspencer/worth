mod phase_three_support;

use phase_three_support::*;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits,
    PhysicalRecoverySourceDenial, PhysicalRecoveryWalIntegrityObservationOutcome,
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
        PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
    );
    assert!(matches!(
        selected.wal_integrity_observations()[1].outcome(),
        PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
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
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::WalInventory);
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(denial, PhysicalRecoverySourceDenial::WalIntegrity(_))));
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
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::WalInventory);
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
}

#[test]
fn empty_terminal_segment_keeps_c8_and_c9_counters_distinct() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let path = root
        .join("families")
        .join("wal")
        .join("segment-1-generation-1.wal");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, []).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.wal_frames, 0);
    assert_eq!(counters.trailing_empty_wal_residue, 1);
    assert_eq!(counters.torn_suffix_frames, 0);
    assert_eq!(counters.wal_corruption_denials, 0);
    assert_eq!(counters.wal_integrity_attempts, 1);
    assert_eq!(counters.wal_integrity_admissions, 0);
    assert_eq!(counters.wal_integrity_rejections, 1);
    assert_eq!(counters.wal_owner_projections, 0);
    assert_eq!(counters.wal_owner_decoder_entries, 0);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.wal_integrity_observations().len(), 1);
    assert!(matches!(
        selected.wal_integrity_observations()[0].outcome(),
        PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
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
    let limits = PhysicalRecoveryLimits::admit(declaration).unwrap();

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
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(limit.dimension, PhysicalRecoveryLimitDimension::WalFrames);
    assert_eq!((limit.observed, limit.admitted), (2, 1));
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

#[test]
fn planning_denial_preserves_alternate_success_wal_observations() {
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
            "planning-complete",
            b"complete",
        );
    let (second_path, second) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            2,
            3,
            4,
            "planning-interrupted",
            b"interrupted",
        );
    std::fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    std::fs::write(first_path, first).unwrap();
    std::fs::write(second_path, &second[..37]).unwrap();
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.redo_bytes = 1;
    let limits = PhysicalRecoveryLimits::admit(declaration).unwrap();

    let selected = admitted_recovery_with_limits(&root, limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(selected.wal_integrity_observations().len(), 2);
    let blocked = expect_blocked(selected.plan().err().unwrap());
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::BindingFreshness);
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
    assert!(matches!(
        blocked.evidence().integrity_observations.wal()[1].outcome(),
        PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
    ));
}
