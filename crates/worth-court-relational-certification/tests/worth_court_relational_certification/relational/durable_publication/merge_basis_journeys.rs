use super::branch_history_publications::*;
use super::merge_proof_journey::*;
use worth_relational::facade::{history::*, merge::*, transactions::MergeExecutionSummary};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn merge_branch_basis_matches_planning_artifact_and_history_scope_authority() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let request = MergePlanningRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );

    let direct_basis = actual_basis(&runtime, "feature", "main");
    let bound = runtime.bind_merge_planning_request(request).unwrap();
    let artifact = runtime
        .merge()
        .inspect_planning_scope(execution, bound)
        .unwrap();

    assert_exact_branch_basis(&artifact.branch_basis, &direct_basis);
    assert_eq!(
        artifact.ancestry.merge_base_commit_id,
        artifact.branch_basis.merge_base().commit().commit_id
    );
    assert_eq!(
        artifact.branch_basis.merge_base().commit().commit_id,
        artifact.merge_base().commit().commit_id
    );
}
#[test]
fn merge_branch_basis_survives_published_merge_outcome_and_durability_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
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
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .unwrap();
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("live merge envelope");
    let live_authority = live_envelope
        .merge_execution_authority
        .clone()
        .expect("live merge authority");

    let (_recovery, recovered) = checkpoint_and_recover(execution, &runtime, &medium);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");
    let recovered_authority = recovered_envelope
        .merge_execution_authority
        .clone()
        .expect("recovered merge authority");

    assert_exact_branch_basis(
        &merge.execution_summary.branch_basis,
        &live_authority.execution_summary.branch_basis,
    );
    assert_exact_branch_basis(
        &merge.execution_summary.branch_basis,
        &recovered_authority.execution_summary.branch_basis,
    );
    assert_summary_projects_exact_basis(&merge.execution_summary);
    assert_summary_projects_exact_basis(&live_authority.execution_summary);
    assert_summary_projects_exact_basis(&recovered_authority.execution_summary);
    assert_eq!(live_authority, recovered_authority);
}
fn assert_exact_branch_basis(
    actual: &RelationalMergeBranchBasis,
    expected: &RelationalMergeBranchBasis,
) {
    assert_eq!(actual, expected);
    assert_eq!(actual.basis_digest(), expected.basis_digest());
    assert_eq!(actual.source_branch(), expected.source_branch());
    assert_eq!(actual.target_branch(), expected.target_branch());
    assert_eq!(actual.source_head(), expected.source_head());
    assert_eq!(actual.target_head(), expected.target_head());
    assert_eq!(
        actual.merge_base().rule(),
        MergeBaseSelectionRule::MaxCommitIdCommonAncestor
    );
    assert_eq!(actual.merge_base().rule(), expected.merge_base().rule());
    assert_eq!(actual.merge_base().commit(), expected.merge_base().commit());
    assert_eq!(
        actual.merge_base().supporting_left_ancestors(),
        expected.merge_base().supporting_left_ancestors()
    );
    assert_eq!(
        actual.merge_base().supporting_right_ancestors(),
        expected.merge_base().supporting_right_ancestors()
    );
}
fn assert_summary_projects_exact_basis(summary: &MergeExecutionSummary) {
    assert_eq!(
        summary.target_head_commit_id,
        summary.branch_basis.target_head().commit_id
    );
    assert_eq!(
        summary.source_head_commit_id,
        summary.branch_basis.source_head().commit_id
    );
    assert_eq!(
        summary.merge_base_commit_id,
        summary.branch_basis.merge_base().commit().commit_id
    );
}
#[test]
fn missing_merge_source_refuses_at_owner_binding_before_planning() {
    let host_request=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    assert!(
        matches!(runtime.bind_merge_planning_request(MergePlanningRequest::new(BranchId("main".into()),BranchId("missing".into()),MergeIntent::ReconcileIntoTarget)),Err(RelationalMergeRequestBindingDenial::UnknownBranch(branch_id)) if branch_id==BranchId("missing".into()))
    );
}
#[test]
fn restored_historical_basis_and_stale_live_basis_differ_from_current_truth() {
    let host_request=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let stale = actual_basis(&runtime, "feature", "main");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-drift",
        BranchId("feature".into()),
    );
    let current = actual_basis(&runtime, "feature", "main");
    assert_ne!(stale.basis_digest(), current.basis_digest());
    let merge = execute_merge(execution, &runtime);
    let live = published_merge_authority(execution, &runtime, merge.commit.commit.commit_id);
    let (_, recovered) = checkpoint_and_recover(execution, &runtime, &medium);
    let restored = published_merge_authority(execution, &recovered, merge.commit.commit.commit_id);
    assert_eq!(live, restored);
    for (runtime, authority) in [(&runtime, &live), (&recovered, &restored)] {
        let now = actual_basis(runtime, "feature", "main");
        assert_ne!(
            authority.execution_summary.branch_basis.basis_digest(),
            now.basis_digest()
        );
        assert_eq!(authority.execution_summary.branch_basis, current);
    }
}
