//! Original merge parentage, currentness and typed authority survive source recovery.
use super::branch_history_publications::{fork, transaction};
use super::branch_history_recovery::current_head;
use super::commit_meaning::assert_durable_commit_meaning_eq;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        RecoveryVerificationMode, RecoveryVerificationMode as RecoveryMode,
        RelationalCanonicalCommitArtifact as Artifact,
    },
    history::BranchId,
    merge::{MergeExecutionRequest, MergeIntent},
    runtime::RelationalRuntime,
    transactions::*,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime, field_patch},
};
#[test]
fn durability_contract_recovery_preserves_merge_parent_order() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let main = create_entity_outcome(execution, &runtime, "main");
    fork(execution, &runtime, "feature");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_feature(execution, &runtime);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(execution, merge.commit.commit_id)
        .unwrap()
        .unwrap();

    assert_eq!(
        recovered_merge.commit.parents,
        vec![main.commit.commit_id, feature.commit.commit_id]
    );
    assert_eq!(
        recovered_merge.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );
    assert_eq!(
        recovered_merge.merge_base_commits,
        vec![main.commit.commit_id]
    );
}

#[test]
fn recovery_applies_merge_currentness_once() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_feature(execution, &runtime);
    let expected = runtime
        .branch_reference_state(&BranchId("main".to_string()))
        .expect("main state after empty-intent merge");
    let expected_target = expected
        .observation()
        .target()
        .as_basis()
        .expect("empty-intent merge still publishes a truth target");

    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let actual = recovered
        .branch_reference_state(&BranchId("main".to_string()))
        .expect("main state after recovery");
    let actual_target = actual
        .observation()
        .target()
        .as_basis()
        .expect("recovered empty-intent merge still has a truth target");

    assert_eq!(
        expected.observation().generation(),
        actual.observation().generation(),
        "recovery must apply the merge currentness transition exactly once"
    );
    assert_eq!(expected.truth_version(), actual.truth_version());
    assert_eq!(
        expected_target.selected_commit_id(),
        actual_target.selected_commit_id()
    );
    assert_eq!(expected_target.version_id(), actual_target.version_id());
    assert_eq!(
        merge.commit.commit_id.0,
        expected_target.selected_commit_id()
    );
}

#[test]
fn durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
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

    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");

    let source_artifact = medium
        .offered()
        .into_iter()
        .find(|artifact| artifact.commit_id() == merge.commit.commit.commit_id)
        .unwrap();
    let source = Artifact::readmit(source_artifact.as_bytes(), execution).unwrap();
    assert!(
        source.envelope().diagnostics_summary.entries.is_empty(),
        "the actual recovery source supplies no diagnostics"
    );
    assert_durable_commit_meaning_eq(source.envelope(), &recovered_merge);
    assert!(recovered_merge.merge_execution_authority.is_some());
    assert_eq!(
        current_head(&recovered, &BranchId("main".to_string()))
            .expect("main head after recovery")
            .commit_id,
        merge.commit.commit.commit_id
    );
}

fn create_entity_outcome_on_branch(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    branch: BranchId,
) -> CommitResult {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(
        WorkerIntentBatch::new(format!("batch-{key}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: worth_relational::facade::identity::KindId(1),
                client_key: key.into(),
                fields: field_patch("name", "name", key),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    txn.commit(execution, runtime).unwrap()
}
fn merge_feature(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> CommitResult {
    let request = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            BranchId("feature".into()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let prepared = runtime.prepare_merge_execution(execution, request).unwrap();
    runtime
        .execute_prepared_merge(execution, prepared)
        .unwrap()
        .commit
}
