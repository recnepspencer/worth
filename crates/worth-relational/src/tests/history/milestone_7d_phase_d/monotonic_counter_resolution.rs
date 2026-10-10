use super::*;

#[test]
fn built_in_monotonic_counter_merge_is_auto_resolved_with_inline_value_and_recovery_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::MonotonicCounter,
    );
    let entity = create_entity_with_aspect_fields(
        &mut runtime,
        "counter",
        crate::tests::support::aspect_field_patch_from_values([(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            i64_counter_value(0),
        )]),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::aspect_field_patch_from_values([(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            i64_counter_value(10),
        )]),
        BranchId("main".to_string()),
    );
    create_branch_from_main(&runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::aspect_field_patch_from_values([(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            i64_counter_value(15),
        )]),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &mut runtime,
        entity,
        crate::tests::support::aspect_field_patch_from_values([(
            crate::tests::support::aspect_key("value"),
            crate::tests::support::field_key("value"),
            i64_counter_value(13),
        )]),
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
    let live_policy_row = live_artifact.policy_resolution.records[0]
        .aspect_resolutions
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("value").unwrap())
        .expect("live policy row");
    assert_eq!(
        live_policy_row.resolved_value_strategy,
        Some(MergeResolvedAspectValueStrategy::InlineAspectValue(
            worth_foundational::facade::AspectValue::Int64(18)
        )),
        "monotonic-counter policy row: {live_policy_row:?}"
    );
    let value_lookup_counters = runtime.performance_access().counters();
    assert_eq!(
        value_lookup_counters.merge_policy_value_source_state_hits,
        1
    );
    assert_eq!(
        value_lookup_counters.merge_policy_value_target_state_hits,
        1
    );
    assert_eq!(value_lookup_counters.merge_policy_value_base_state_hits, 0);
    assert_eq!(
        value_lookup_counters.merge_policy_value_base_patch_authority_hits,
        1
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
        .expect("prepared counter merge");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed counter merge");
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
        "18"
    );
    let live_envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("live merge envelope");

    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::MonotonicCounter,
        )
    });
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(live_commit_id)
        .expect("recovered merge envelope");
    assert_eq!(
        read_entity_aspect_field_display(
            execution,
            &recovered,
            &BranchId("main".to_string()),
            entity,
            AspectKey::new("value").unwrap(),
            field_key("value")
        ),
        "18"
    );
    assert_eq!(
        live_envelope.diagnostics_summary,
        recovered_envelope.diagnostics_summary
    );
    let summary_entry = live_envelope
        .diagnostics_summary
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
        .expect("merge execution summary entry");
    assert_eq!(
        diagnostic_field(summary_entry, "execution_digest"),
        &RelationalDiagnosticValue::String(merge.execution_summary.execution_digest.clone())
    );
    assert_eq!(merge.execution_summary.executed_record_count, 1);
    assert!(
        merge.commit.patch().iter().any(|record| record
            .authoritative_changed_aspects()
            .iter()
            .any(|aspect| aspect == &AspectKey::new("value").unwrap())),
        "counter merge patch should tag the declared aspect key: {:?}",
        merge.commit.patch()
    );
}
