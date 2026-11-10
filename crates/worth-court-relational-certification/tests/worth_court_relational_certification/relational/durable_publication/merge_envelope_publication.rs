//! A prepared merge publishes its ordered multi-parent commit through the canonical envelope.
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use super::branch_history_recovery::current_head;
use super::merge_feature_execution::execute_feature_into_main_merge;
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    diagnostics::{DiagnosticCode, RelationalDiagnosticsProfile},
    history::BranchId,
    merge::{MergeExecutionRequest, MergeIntent},
    runtime::RelationalRuntimeApi,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, schema},
};

#[test]
fn execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (_medium, runtime, merge, main_head_commit_id, feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let replay = runtime.replay();

    assert_eq!(merge.commit.merge_parent_count(), 1);
    assert_eq!(
        merge.commit.commit.parents,
        vec![main_head_commit_id, feature_head_commit_id]
    );
    assert_eq!(
        merge.execution_summary.target_head_commit_id,
        main_head_commit_id
    );
    assert_eq!(
        merge.execution_summary.source_head_commit_id,
        feature_head_commit_id
    );
    assert_eq!(merge.execution_summary.executed_record_count, 1);

    let envelope = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("canonical merge envelope");
    assert_eq!(envelope.commit.parents, merge.commit.commit.parents);
    assert_eq!(
        envelope.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );
    assert_eq!(
        current_head(&runtime, &BranchId("main".to_string()))
            .expect("main branch head")
            .commit_id,
        merge.commit.commit.commit_id
    );
    assert_eq!(
        current_head(&runtime, &BranchId("feature".to_string()))
            .expect("feature branch head")
            .commit_id,
        feature_head_commit_id
    );
}

#[test]
fn execute_prepared_merge_preserves_reserved_summary_when_optional_diagnostics_budget_is_zero() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .diagnostics(RelationalDiagnosticsProfile {
            max_entries_per_artifact: 0,
            ..RelationalRuntimeProfile::CertificationCore.default_diagnostics_profile()
        })
        .schema_registry(schema())
        .build_durable(medium.attachment())
        .unwrap();
    create_entity_outcome(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .unwrap(),
        )
        .expect("prepared merge execution");

    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("canonical merge envelope");

    assert_eq!(envelope.diagnostics_summary.entries.len(), 1);
    assert_eq!(
        envelope.diagnostics_summary.entries[0].code,
        DiagnosticCode::MergeExecutionPublished
    );
}
