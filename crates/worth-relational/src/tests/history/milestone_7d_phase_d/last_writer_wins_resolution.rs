use super::*;

#[test]
fn built_in_last_writer_wins_reject_retained_envelope_is_stable_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::LastWriterWins,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &mut runtime,
        "shared",
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "base",
        ),
    );
    create_branch_from_main(&runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "main-change",
        ),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "feature-change",
        ),
        BranchId("feature".to_string()),
    );

    runtime.performance_access().reset_counters();
    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
        )
        .expect("live planning artifact");
    let live_record = live_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("live lowered record");
    let live_policy_row = live_artifact.policy_resolution.records[0]
        .aspect_resolutions
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("value").unwrap())
        .expect("live policy row");

    assert_eq!(
        live_policy_row.applied_policy,
        Some(AspectMergePolicyKind::LastWriterWins)
    );
    assert_eq!(
        live_policy_row.decision_boundary,
        MergePolicyDecisionBoundary::Reject {
            class: crate::facade::merge::MergePolicyRejectClass::LastWriterWinsCausalConflict,
        },
        "last-writer-wins policy row: {live_policy_row:?}"
    );
    assert_eq!(live_policy_row.resolved_value_strategy, None);
    assert!(matches!(
        live_record.record_decision,
        crate::facade::merge::LoweredRecordDecision::Reject(_)
    ));

    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::LastWriterWins,
        )
    });
    let recovered_artifact = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
        )
        .expect("recovered planning artifact");

    assert_eq!(
        live_artifact.digest_basis.policy,
        recovered_artifact.digest_basis.policy
    );
    assert_eq!(
        live_artifact.digest_basis.lowered_plan,
        recovered_artifact.digest_basis.lowered_plan
    );
}

#[test]
fn built_in_last_writer_wins_auto_resolution_is_stable_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::LastWriterWins,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &mut runtime,
        "shared",
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "base",
        ),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "main-change",
        ),
        BranchId("main".to_string()),
    );
    create_branch_from_main(&runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            "feature-change",
        ),
        BranchId("feature".to_string()),
    );

    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
        )
        .expect("live planning artifact");
    let live_record = live_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("live lowered record");
    let live_policy_row = live_artifact.policy_resolution.records[0]
        .aspect_resolutions
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("value").unwrap())
        .expect("live policy row");

    assert_eq!(
        live_policy_row.decision_boundary,
        MergePolicyDecisionBoundary::AutoResolved
    );
    assert_eq!(
        live_policy_row.resolved_value_strategy,
        Some(MergeResolvedAspectValueStrategy::SourceVisibleValue)
    );
    assert!(matches!(
        live_record.record_decision,
        crate::facade::merge::LoweredRecordDecision::Execute(_)
    ));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared last-writer-wins merge");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed last-writer-wins merge");
    let live_commit_id = merge.commit.commit.commit_id;
    assert_eq!(
        read_entity_aspect_field_display(
            execution,
            &runtime,
            &BranchId("main".to_string()),
            entity,
            AspectKey::new("value").unwrap(),
            field_key("value")
        ),
        "feature-change"
    );

    let live_envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .expect("live merge envelope");
    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::LastWriterWins,
        )
    });
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, live_commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .expect("recovered merge envelope");

    assert_eq!(
        live_envelope.diagnostics_summary,
        recovered_envelope.diagnostics_summary
    );
    assert_eq!(
        read_entity_aspect_field_display(
            execution,
            &recovered,
            &BranchId("main".to_string()),
            entity,
            AspectKey::new("value").unwrap(),
            field_key("value")
        ),
        "feature-change"
    );
}

#[test]
fn auto_resolved_merge_reads_pinned_visible_value_through_declared_aspect_binding() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("display_name").unwrap(),
        field_key("display"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &mut runtime,
        "identity",
        string_aspect_field_patch_for_target(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "base",
        ),
    );
    create_branch_from_main(&runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        string_aspect_field_patch_for_target(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "main-change",
        ),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        string_aspect_field_patch_for_target(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "feature-change",
        ),
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared binding-native merge");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed binding-native merge");

    assert_eq!(
        read_entity_aspect_field_display(
            execution,
            &runtime,
            &BranchId("main".to_string()),
            entity,
            AspectKey::new("display_name").unwrap(),
            field_key("display")
        ),
        "feature-change"
    );
}
