use super::*;

#[test]
fn page_failures_retain_distinct_exact_phase_four_read_counters() {
    let invalid_root = prepare_ordinary_recovery_root("c8-phase4-invalid-membership");
    let invalid_selected = selected_ordinary_recovery(invalid_root.path());
    let generation = invalid_selected.root_generation();
    let membership_prefix = format!("segments-{generation:016x}-block-");
    let membership = only_artifact(
        invalid_root
            .path()
            .join("families/records/segment-manifests"),
        |name| name.starts_with(&membership_prefix),
    );
    std::fs::write(membership, [0_u8; 32]).unwrap();
    let invalid = match invalid_selected.plan() {
        Ok(_) => panic!("an invalid membership block cannot form a plan"),
        Err(outcome) => expect_blocked(outcome),
    };
    // Six checkpoint attempts survive discovery alongside the four routing
    // attempts up to the damaged membership block.
    assert_eq!(invalid.evidence().integrity_observation_count(), 10);

    let page_root = prepare_ordinary_recovery_root("c8-phase4-invalid-page");
    let page_selected = selected_ordinary_recovery(page_root.path());
    let source_generation = page_selected.root_generation() - 1;
    let page_suffix = format!("-{source_generation:016x}.pages");
    let page = only_artifact(page_root.path().join("families/records/segments"), |name| {
        name.ends_with(&page_suffix)
    });
    let page_length = std::fs::metadata(&page).unwrap().len() as usize;
    std::fs::write(page, vec![0_u8; page_length]).unwrap();
    let invalid_page = match page_selected.plan() {
        Ok(_) => panic!("an invalid selected page cannot form a plan"),
        Err(outcome) => expect_blocked(outcome),
    };

    let invalid_counters = invalid.evidence().planning_counters.unwrap();
    let page_counters = invalid_page.evidence().planning_counters.unwrap();
    assert_eq!(invalid_counters.page_extent_reads(), 2);
    assert_eq!(invalid_counters.page_extent_bytes(), 208);
    assert_eq!(
        invalid_counters.freshness_retained() + invalid_counters.freshness_expired(),
        3
    );
    assert_eq!(invalid_counters.fate_counts(), [1, 0, 0, 2]);
    assert_eq!(page_counters.page_extent_reads(), 7);
    assert_eq!(page_counters.page_extent_bytes(), 17_328);
    assert_eq!(
        page_counters.freshness_retained() + page_counters.freshness_expired(),
        3
    );
    assert_eq!(page_counters.fate_counts(), [1, 0, 0, 2]);
    assert_eq!(page_counters.page_extent_integrity_attempts(), 1);
    assert_eq!(page_counters.page_extent_integrity_admissions(), 0);
    assert_eq!(page_counters.page_extent_integrity_rejections(), 1);
    assert_eq!(page_counters.page_extent_owner_projections(), 0);
    assert_eq!(page_counters.page_extent_owner_decoders(), 0);
    let rejected = invalid_page
        .evidence()
        .integrity_observations()
        .last()
        .unwrap();
    assert_eq!(
        rejected.scope().artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PageFrame,
    );
    assert!(matches!(
        rejected.outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(_),
    ));
    assert_eq!(invalid.recovery_effects(), 0);
    assert_eq!(invalid_page.recovery_effects(), 0);
}

#[test]
fn redo_denial_retains_completed_phase_four_read_counters() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-redo-denial-counters");
    let selected = selected_ordinary_recovery(retained_root.path());
    let source_generation = selected.root_generation() - 1;
    let page_suffix = format!("-{source_generation:016x}.pages");
    let page = only_artifact(
        retained_root.path().join("families/records/segments"),
        |name| name.ends_with(&page_suffix),
    );
    let mut bytes = std::fs::read(&page).unwrap();
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(3),
    )
    .unwrap();
    std::fs::write(page, bytes).unwrap();

    let blocked = match selected.plan() {
        Ok(_) => panic!("an equal-LSN wrong-digest page cannot form a plan"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::Redo(
            PhysicalRedoPlanningDenial::PageDigestMismatch,
        ))
    );
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(counters.page_extent_reads(), 7);
    assert!(counters.page_extent_bytes() > 0);
    assert_eq!(counters.fate_counts(), [1, 0, 0, 2]);
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn staging_denial_retains_every_completed_planning_stage_counter() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-staging-denial-counters");
    let mut declaration = ordinary_recovery_declaration(4_096);
    declaration.staging_bytes = 3_276_799;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
    let selected = admitted_recovery_with_limits(retained_root.path(), limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();

    let blocked = match selected.plan() {
        Ok(_) => panic!("one-over staging allocation must block before execution"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::Cost(
            RecoveryPlanCostDenial::StagingBytes,
        ))
    );
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::StagingBytes
    );
    assert_eq!(limit.observed, 3_276_800);
    assert_eq!(limit.admitted, 3_276_799);
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(counters.page_extent_reads(), 7);
    assert_eq!(counters.page_extent_bytes(), 17_328);
    assert_eq!(counters.redo_records(), 2);
    assert_eq!(counters.redo_targets(), 2);
    assert_eq!(counters.redo_apply(), 1);
    assert_eq!(counters.redo_skip_page_lsn(), 1);
    assert_eq!(counters.fate_counts(), [1, 1, 0, 1]);
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn late_binding_limit_retains_sampled_freshness_without_media_reads() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-binding-limit-counters");
    let mut declaration = ordinary_recovery_declaration(4_096);
    declaration.operation_bindings = 2;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
    let selected = admitted_recovery_with_limits(retained_root.path(), limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();

    let blocked = match selected.plan() {
        Ok(_) => panic!("the third operation binding must exceed the admitted limit"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::BindingFreshness(
            StoreRecoveryBindingSampleDenial::OperationBindingLimit,
        ))
    );
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::OperationBindings
    );
    assert_eq!((limit.observed, limit.admitted), (3, 2));
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(
        counters.freshness_retained() + counters.freshness_expired(),
        2
    );
    assert_eq!(counters.page_extent_reads(), 0);
    assert_eq!(counters.fate_counts(), [0; 4]);
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn redo_byte_limit_reports_the_exact_non_unit_crossing_without_media_reads() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-redo-byte-counters");
    let mut declaration = ordinary_recovery_declaration(4_096);
    declaration.redo_bytes = 1;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
    let selected = admitted_recovery_with_limits(retained_root.path(), limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();

    let blocked = match selected.plan() {
        Ok(_) => panic!("the first canonical redo member must exceed one byte"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::BindingFreshness(
            StoreRecoveryBindingSampleDenial::RedoByteLimit,
        ))
    );
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(limit.dimension, PhysicalRecoveryLimitDimension::RedoBytes);
    assert_eq!(limit.admitted, 1);
    assert!(limit.observed > limit.admitted + 1);
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(counters.page_extent_reads(), 0);
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn redo_admission_denial_retains_sampled_freshness_and_reconciled_fates() {
    let retained_root = prepare_ordinary_recovery_root("c8-phase4-redo-admission-counters");
    let mut declaration = ordinary_recovery_declaration(4_096);
    declaration.redo_targets = 1;
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
    let selected = admitted_recovery_with_limits(retained_root.path(), limits)
        .discover()
        .unwrap()
        .select()
        .unwrap();

    let blocked = match selected.plan() {
        Ok(_) => panic!("the second canonical redo target must exceed the admitted limit"),
        Err(outcome) => expect_blocked(outcome),
    };
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(PhysicalRecoveryPlanningDenial::Redo(
            PhysicalRedoPlanningDenial::TargetLimit,
        ))
    );
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(counters.page_extent_reads(), 0);
    assert_eq!(
        counters.freshness_retained() + counters.freshness_expired(),
        3
    );
    assert_eq!(counters.fate_counts(), [1, 0, 0, 2]);
    assert_eq!(blocked.recovery_effects(), 0);
}
