#[test]
fn corrupt_published_selector_blocks_fresh_reopen_before_the_root_read() {
    let retained_root =
        crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-corrupt-selector");
    let published = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    let selector = retained_root
        .path()
        .join("families/records")
        .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name());
    std::fs::write(
        selector,
        [0_u8; worth_store_physical_format::ROOT_SELECTOR_BYTES],
    )
    .expect("corrupt current selector");

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = published.reopen()
    else {
        panic!("corrupt namespace-durable selector requires indeterminate reopen evidence")
    };
    let failure = outcome
        .reopen_failure()
        .expect("fresh-reopen failure evidence");
    assert_eq!(failure.counters().selector_reads_completed, 1);
    assert_eq!(failure.counters().root_reads_completed, 0);
    assert_eq!(
        failure.denial().kind(),
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind::InvalidSelector
    );
    assert!(outcome.recovery_effects() > 0);
}

#[test]
fn corrupt_published_root_retains_both_completed_fresh_reads() {
    let retained_root =
        crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-corrupt-root");
    let published = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    let generation = published.publication_expectation().staging_generation();
    let root = retained_root
        .path()
        .join("families/records")
        .join("roots")
        .join(
            worth_store_physical_format::RecordArtifactFile::RootManifest { generation }
                .file_name(),
        );
    std::fs::write(root, [0_u8; 64]).expect("corrupt published root");

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = published.reopen()
    else {
        panic!("corrupt namespace-durable root requires indeterminate reopen evidence")
    };
    let failure = outcome
        .reopen_failure()
        .expect("fresh-reopen failure evidence");
    assert_eq!(failure.counters().selector_reads_completed, 1);
    assert_eq!(failure.counters().root_reads_completed, 1);
    assert_eq!(
        failure.denial().kind(),
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind::InvalidRoot
    );
    assert!(outcome.recovery_effects() > 0);
}
