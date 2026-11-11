//! A declared monotonic-counter merge resolves inline, executes and survives Memory reconstruction.
use super::branch_history_publications::fork;
use super::merge_execution_authoring::{
    aspect_field_patch_from_values, create_entity_with_aspect_fields, i64_counter_value,
    update_entity_aspect_fields_on_branch,
};
use super::merge_execution_observation::{diagnostic_field, read_entity_aspect_field_display};
use super::merge_execution_recovery::checkpoint_and_recover_memory;
use super::merge_execution_schema::runtime_with_aspect_field_merge_policy;
use super::merge_proof_journey::merge_request;
use worth_foundational::facade::{AspectKey, AspectValue};
use worth_relational::facade::{
    diagnostics::{DiagnosticCode, RelationalDiagnosticValue},
    history::BranchId,
    merge::{AspectMergePolicyKind, MergeResolvedAspectValueStrategy},
};
use worth_testkit_relational::aspect_storage::definition::{aspect_key, field_key};

#[test]
fn built_in_monotonic_counter_merge_is_auto_resolved_with_inline_value_and_recovery_parity() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_aspect_field_merge_policy(
        AspectKey::new("value").unwrap(),
        field_key("value"),
        AspectMergePolicyKind::MonotonicCounter,
    );
    let entity = create_entity_with_aspect_fields(
        execution,
        &runtime,
        "counter",
        aspect_field_patch_from_values([(
            aspect_key("value"),
            field_key("value"),
            i64_counter_value(0),
        )]),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        aspect_field_patch_from_values([(
            aspect_key("value"),
            field_key("value"),
            i64_counter_value(10),
        )]),
        BranchId("main".to_string()),
    );
    fork(execution, &runtime, "feature");
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        aspect_field_patch_from_values([(
            aspect_key("value"),
            field_key("value"),
            i64_counter_value(15),
        )]),
        BranchId("main".to_string()),
    );
    update_entity_aspect_fields_on_branch(
        execution,
        &runtime,
        entity,
        aspect_field_patch_from_values([(
            aspect_key("value"),
            field_key("value"),
            i64_counter_value(13),
        )]),
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
    let live_policy_row = live_artifact.policy_resolution.records[0]
        .aspect_resolutions
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("value").unwrap())
        .expect("live policy row");
    assert_eq!(
        live_policy_row.resolved_value_strategy,
        Some(MergeResolvedAspectValueStrategy::InlineAspectValue(
            AspectValue::Int64(18)
        )),
        "monotonic-counter policy row: {live_policy_row:?}"
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
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
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("live merge envelope");

    let (_recovery, recovered) = checkpoint_and_recover_memory(
        execution,
        &runtime,
        runtime_with_aspect_field_merge_policy(
            AspectKey::new("value").unwrap(),
            field_key("value"),
            AspectMergePolicyKind::MonotonicCounter,
        ),
    );
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, live_commit_id)
        .unwrap()
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
