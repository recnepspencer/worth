//! Merge replay continuity from typed merge authority when diagnostics are absent.
//! Fixture adaptation (C2.4): restored from 622f101e99 on the Memory runtime.
//! Instead of rewriting a commit inside the retired in-crate test medium, the
//! canonical commit artifact for the merge commit is encoded, readmitted,
//! inspected with the original authority take/restore allocation-inventory
//! callback, resealed, and replayed through the owner's recovery plan.
use super::*;
#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::migration::ReadmittedCanonicalCommit;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use worth_relational_branch::internal_port::history::data::canonical_commit_allocation::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;

#[test]
fn durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge execution");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");

    let mut plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let merge_index = plan
        .tail_log
        .iter()
        .position(|entry| entry.commit_id() == merge.commit.commit.commit_id)
        .expect("merge commit is in recovery tail");
    let source_artifact = RelationalCanonicalCommitArtifact::from_positioned(
        plan.tail_log[merge_index].positioned(),
        execution.allocation_policy(),
    )
    .unwrap();
    let readmitted =
        RelationalCanonicalCommitArtifact::readmit(source_artifact.as_bytes(), execution).unwrap();
    let position = readmitted.position();
    let mut merge_entry = copy_original_for_test(&readmitted.envelope(), execution);
    {
        let body = merge_entry.body_mut_for_test();
        assert!(body.merge_execution_authority.is_some());
        let populated_nested_bytes = body.allocation_inventory().authoritative_nested_bytes;
        let authority = body.merge_execution_authority.take().unwrap();
        let omitted_nested_bytes = body.allocation_inventory().authoritative_nested_bytes;
        body.merge_execution_authority = Some(authority);
        assert!(populated_nested_bytes > omitted_nested_bytes);
        assert!(
            body.diagnostics_summary.entries.is_empty(),
            "diagnostics never reach the durable artifact"
        );
    }
    let resealed_commit =
        ReadmittedCanonicalCommit::readmit_current(position, merge_entry).unwrap();
    let resealed_artifact = RelationalCanonicalCommitArtifact::from_positioned(
        resealed_commit.positioned(),
        execution.allocation_policy(),
    )
    .unwrap();
    let source =
        RelationalCanonicalCommitArtifact::readmit(resealed_artifact.as_bytes(), execution)
            .unwrap();
    assert!(
        source.envelope().diagnostics_summary.entries.is_empty(),
        "the actual recovery source supplies no diagnostics"
    );
    plan.tail_log[merge_index] = source.clone();

    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let replay = recovered.replay();
    let recovered_merge = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");

    crate::tests::support::assert_durable_commit_meaning_eq(
        execution,
        source.envelope(),
        &recovered_merge,
    );
    assert!(recovered_merge.merge_execution_authority.is_some());
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string()))
            .expect("main head after recovery")
            .commit_id,
        merge.commit.commit.commit_id
    );
}

#[test]
fn durability_contract_rejects_parent_order_tampering_at_source_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let main = create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_commit_from_branches(
        execution,
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );

    medium.rewrite_commit_envelope(execution, merge.commit.commit_id, |merge_entry| {
        assert_eq!(
            merge_entry.commit.parents,
            vec![main.commit.commit_id, feature.commit.commit_id]
        );
        merge_entry.commit.parents.reverse();
    });

    let error = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert_eq!(error.history_drift_class, None);
    assert!(error
        .detail
        .contains("PositionAdmission(SemanticResult(ReceiptMismatch))"));
}

#[test]
fn durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "main");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge execution");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");

    medium.rewrite_commit_envelope(execution, merge.commit.commit.commit_id, |merge_entry| {
        assert!(merge_entry.merge_execution_authority.is_some());
        merge_entry.merge_execution_authority = None;
    });

    let plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert_eq!(error.history_drift_class, None);
    assert!(error
        .detail
        .contains("failed to reconstruct merge execution summary"));
}
