//! Declared last-writer-wins and prefer-richer merges resolve, execute and survive Memory reconstruction.
use super::branch_history_publications::fork;
use super::merge_execution_authoring::{
    create_entity_with_aspect_fields, single_string_aspect_field_patch,
    update_entity_aspect_fields_on_branch,
};
use super::merge_execution_observation::read_entity_aspect_field_display;
use super::merge_execution_recovery::checkpoint_and_recover_memory;
use super::merge_execution_schema::runtime_with_aspect_field_merge_policy;
use super::merge_proof_journey::merge_request;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    history::BranchId,
    merge::{
        AspectMergePolicyKind, LoweredRecordDecision, MergePolicyDecisionBoundary,
        MergePolicyRejectClass, MergeResolvedAspectValueStrategy,
    },
    transactions::RecordRef,
};
use worth_testkit_relational::aspect_storage::definition::{aspect_key, field_key};

#[test]
fn built_in_last_writer_wins_reject_retained_envelope_is_stable_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::LastWriterWins,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &runtime,
        "shared",
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "base"),
    );
    fork(execution, &runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "main-change"),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "feature-change"),
        BranchId("feature".to_string()),
    );

    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
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
            class: MergePolicyRejectClass::LastWriterWinsCausalConflict,
        },
        "last-writer-wins policy row: {live_policy_row:?}"
    );
    assert_eq!(live_policy_row.resolved_value_strategy, None);
    assert!(matches!(
        live_record.record_decision,
        LoweredRecordDecision::Reject(_)
    ));

    let (_recovery, recovered) = checkpoint_and_recover_memory(
        execution,
        &runtime,
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::LastWriterWins,
        ),
    );
    let recovered_artifact = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            recovered
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::LastWriterWins,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &runtime,
        "shared",
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "base"),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "main-change"),
        BranchId("main".to_string()),
    );
    fork(execution, &runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(aspect_key("value"), field_key("value"), "feature-change"),
        BranchId("feature".to_string()),
    );

    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
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
        LoweredRecordDecision::Execute(_)
    ));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
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
        .expect("live merge envelope");
    let (_recovery, recovered) = checkpoint_and_recover_memory(
        execution,
        &runtime,
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::LastWriterWins,
        ),
    );
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, live_commit_id)
        .unwrap()
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("display_name").unwrap(),
        field_key("display"),
        AspectMergePolicyKind::PreferRicher,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &runtime,
        "identity",
        single_string_aspect_field_patch(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "base",
        ),
    );
    fork(execution, &runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "main-change",
        ),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        single_string_aspect_field_patch(
            AspectKey::new("display_name").unwrap(),
            field_key("display"),
            "feature-change",
        ),
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
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
