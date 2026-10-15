use super::{MilestoneFivePointSixUnifiedFacadeCertificationAdapter, UnifiedFacadeFailureClass};

#[test]
fn unified_facade_certification_matrix_contains_required_rows() {
    let matrix = MilestoneFivePointSixUnifiedFacadeCertificationAdapter::
        unified_facade_and_configuration_boundary_test();

    for row in &matrix.rows {
        assert!(!row.control_lane.support_report_digest.is_empty());
        assert_eq!(
            row.control_lane.support_report_digest,
            row.parity_lane.support_report_digest
        );
        assert_eq!(row.control_lane.support_report_generation_count, 1);
        assert_eq!(row.control_lane.capability_lookup_count, 1);
        assert_eq!(row.control_lane.configuration_section_resolution_count, 1);
        assert_eq!(row.control_lane.unsupported_composition_denial_count, 0);
        assert_eq!(row.control_lane.deferred_capability_denial_count, 0);
        assert_eq!(
            row.control_lane.query_context_basis_families.len(),
            row.parity_lane.query_context_basis_families.len()
        );
    }
    let support_sync = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "capability-support-metadata-sync")
        .expect("support metadata sync row should exist");
    assert_eq!(support_sync.control_lane.capability_family, "query_context");

    let preview = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-preview-capability")
        .expect("preview row should exist");
    assert_eq!(preview.control_lane.capability_family, "preview_session");

    let identity = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-identity-evolution-capability")
        .expect("identity evolution row should exist");
    assert_eq!(
        identity.control_lane.capability_family,
        "identity_evolution"
    );
    assert!(!identity
        .control_lane
        .identity_evolution_result_digest
        .is_empty());
    assert!(!identity
        .control_lane
        .identity_evolution_branch_locality_digest
        .is_empty());

    let workflow = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-workflow-capability")
        .expect("workflow row should exist");
    assert_eq!(
        workflow.control_lane.capability_family,
        "workflow_orchestration"
    );

    let historical = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-historical-capability")
        .expect("historical row should exist");
    assert_eq!(
        historical.control_lane.capability_family,
        "historical_evaluation"
    );

    let basis_bundle = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-query-context-basis-result-bundle")
        .expect("basis result bundle row should exist");
    assert!(!basis_bundle.control_lane.basis_result_digest.is_empty());
    assert!(!basis_bundle
        .control_lane
        .query_context_replay_digest
        .is_empty());

    let diff_bundle = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-query-context-diff-result-bundle")
        .expect("diff result bundle row should exist");
    assert!(!diff_bundle.control_lane.diff_result_digest.is_empty());
    assert!(!diff_bundle
        .control_lane
        .query_context_replay_digest
        .is_empty());

    let config_section = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "unified-config-section-explicitness")
        .expect("config section row should exist");
    assert_eq!(config_section.control_lane.capability_lookup_count, 1);
    assert_eq!(
        config_section.hostile_lane.capability_lookup_count, 0,
        "section resolution row should prove section work without capability lookup"
    );
    assert_eq!(
        config_section
            .hostile_lane
            .configuration_section_resolution_count,
        1
    );

    let profile_sync = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "query-context-support-profile-sync")
        .expect("query-context support profile row should exist");
    assert!(!profile_sync
        .control_lane
        .query_context_support_profile_digest
        .is_empty());
    assert_eq!(
        profile_sync.control_lane.query_context_basis_families,
        vec![
            "current_branch_head",
            "branch_head",
            "historical_snapshot",
            "historical_commit",
            "preview_derived_historical",
        ]
    );
    assert_eq!(
        profile_sync.control_lane.query_context_comparison_families,
        vec![
            "branch_to_branch",
            "current_to_historical",
            "historical_to_historical",
            "preview_to_authoritative",
        ]
    );
    assert_eq!(
        profile_sync
            .control_lane
            .query_context_deferred_scope_markers,
        vec![
            "store_backed_historical",
            "store_backed_diff",
            "broad_collection_diff",
        ]
    );

    let identity_profile = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-evolution-support-profile-sync")
        .expect("identity evolution support profile row should exist");
    assert!(!identity_profile
        .control_lane
        .identity_evolution_support_profile_digest
        .is_empty());
    assert_eq!(
        identity_profile
            .control_lane
            .identity_evolution_traversal_families,
        vec![
            "direct_predecessor",
            "direct_successor",
            "direct_replacement",
            "direct_split_successors",
            "direct_merge_successor",
            "branch_local_direct_evolution",
        ]
    );
    assert_eq!(
        identity_profile
            .control_lane
            .identity_evolution_comparison_basis_families,
        vec![
            "branch_to_branch",
            "current_to_historical",
            "historical_to_historical",
            "preview_to_authoritative",
        ]
    );
    assert_eq!(
        identity_profile
            .control_lane
            .identity_evolution_inspector_consumable_classifications,
        vec![
            "identity_summary",
            "authoritative_continuity",
            "advisory_candidates",
            "ambiguity",
            "identity_break",
            "denied",
        ]
    );
    assert_eq!(
        identity_profile
            .control_lane
            .identity_evolution_deferred_scope_markers,
        vec![
            "recursive_traversal",
            "broad_collection_discovery",
            "store_backed_parity",
            "identity_aware_non_inspector_views",
        ]
    );
}

#[test]
fn unified_facade_rejections_preserve_failure_class_honesty() {
    let matrix = MilestoneFivePointSixUnifiedFacadeCertificationAdapter::
        unified_facade_and_configuration_boundary_test();

    let unsupported = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "missing-owning-live-section")
        .expect("missing owning live section rejection row should exist");
    assert_eq!(
        unsupported.hostile_lane.failure_class,
        UnifiedFacadeFailureClass::MissingOwningSection
    );
    assert_eq!(unsupported.hostile_lane.capability_lookup_count, 1);
    assert_eq!(
        unsupported
            .hostile_lane
            .unsupported_composition_denial_count,
        1
    );
    assert_eq!(unsupported.hostile_lane.deferred_capability_denial_count, 0);
    assert_eq!(unsupported.hostile_lane.config_validation_denial_count, 0);

    let invalid_support = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "invalid-workflow-support-posture")
        .expect("invalid workflow support posture rejection row should exist");
    assert_eq!(
        invalid_support.hostile_lane.failure_class,
        UnifiedFacadeFailureClass::InvalidComposedSupportPosture
    );
    assert_eq!(invalid_support.hostile_lane.capability_lookup_count, 1);
    assert_eq!(
        invalid_support
            .hostile_lane
            .unsupported_composition_denial_count,
        1
    );
    assert_eq!(
        invalid_support
            .hostile_lane
            .deferred_capability_denial_count,
        0
    );
    assert_eq!(
        invalid_support.hostile_lane.config_validation_denial_count,
        0
    );

    let deferred = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "deferred-durable-artifacts")
        .expect("deferred durability rejection row should exist");
    assert_eq!(
        deferred.hostile_lane.failure_class,
        UnifiedFacadeFailureClass::DeferredCapability
    );
    assert_eq!(deferred.hostile_lane.capability_lookup_count, 1);
    assert_eq!(
        deferred.hostile_lane.unsupported_composition_denial_count,
        0
    );
    assert_eq!(deferred.hostile_lane.deferred_capability_denial_count, 1);
    assert_eq!(deferred.hostile_lane.config_validation_denial_count, 0);

    let invalid_config = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "invalid-unified-configuration")
        .expect("invalid unified config rejection row should exist");
    assert_eq!(
        invalid_config.hostile_lane.failure_class,
        UnifiedFacadeFailureClass::InvalidConfiguration
    );
    assert_eq!(invalid_config.hostile_lane.capability_lookup_count, 0);
    assert_eq!(
        invalid_config
            .hostile_lane
            .configuration_section_resolution_count,
        5
    );
    assert_eq!(
        invalid_config.hostile_lane.config_validation_denial_count,
        1
    );
    assert_eq!(
        invalid_config
            .hostile_lane
            .unsupported_composition_denial_count,
        0
    );
    assert_eq!(
        invalid_config.hostile_lane.deferred_capability_denial_count,
        0
    );

    let broad_diff = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "broad-collection-diff-denied")
        .expect("broad collection diff rejection row should exist");
    assert_eq!(
        broad_diff.hostile_lane.failure_class,
        UnifiedFacadeFailureClass::QueryContextBroadeningDenied
    );
    assert_eq!(broad_diff.hostile_lane.capability_lookup_count, 1);
    assert_eq!(broad_diff.hostile_lane.deferred_capability_denial_count, 0);
    assert_eq!(broad_diff.hostile_lane.query_context_denial_width, 1);
    assert_eq!(
        broad_diff
            .hostile_lane
            .query_context_broadening_denial_count,
        1
    );
}
