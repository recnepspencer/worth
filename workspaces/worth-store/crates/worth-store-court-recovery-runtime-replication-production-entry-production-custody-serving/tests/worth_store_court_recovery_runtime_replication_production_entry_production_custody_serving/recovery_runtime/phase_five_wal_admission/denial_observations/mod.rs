use super::*;

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
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();

    let selected = admitted_recovery_with_limits(&root, limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(selected.wal_integrity_observations().len(), 2);
    let blocked = expect_blocked(selected.plan().err().unwrap());
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::BindingFreshness)
    );
    assert_eq!(blocked.evidence().integrity_observations.wal().len(), 2);
    assert!(matches!(
        blocked.evidence().integrity_observations.wal()[1].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(_)
    ));
}
