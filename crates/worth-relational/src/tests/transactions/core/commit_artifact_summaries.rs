use crate::facade::runtime::InvariantExecutionPoint;
use crate::facade::transactions::{CommitPhase, CommitTopology, CommitTraceEvent};
use crate::tests::support::*;

#[test]
fn commit_log_records_structural_summary_and_phase_progress() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "logged");
    let structural_summary = outcome.structural_summary();
    let history_summary = outcome.history_summary().unwrap();
    let change_summary = outcome.change_summary().unwrap();
    let aspect_summary = outcome.aspect_summary().unwrap();
    let patch_budget_summary = outcome.patch_budget_summary().unwrap();
    let publication_summary = outcome.publication_summary().unwrap();
    let commit_summary = outcome.commit_summary();

    assert_eq!(
        structural_summary.commit_topology,
        CommitTopology::FlatEntityBatch
    );
    assert!(!structural_summary.invariant_groups.is_empty());
    assert!(!structural_summary.touched_partitions.is_empty());
    assert!(commit_summary.invariant_result_count >= 1);
    assert_eq!(
        commit_summary.structural_summary.as_ref(),
        Some(structural_summary)
    );
    assert_eq!(
        publication_summary.patch_position,
        Some(outcome.patch_position())
    );
    assert_eq!(
        publication_summary.final_snapshot_id,
        Some(outcome.final_snapshot_id())
    );
    assert_eq!(outcome.outcome().history_summary(), Some(history_summary));
    assert_eq!(outcome.outcome().change_summary(), Some(change_summary));
    assert_eq!(outcome.outcome().aspect_summary(), Some(aspect_summary));
    assert_eq!(
        outcome.outcome().patch_budget_summary(),
        Some(patch_budget_summary)
    );
    assert_eq!(
        outcome.outcome().publication_summary(),
        Some(publication_summary)
    );
    assert_eq!(history_summary.parent_count, outcome.commit.parents.len());
    assert_eq!(history_summary.target_branch, outcome.commit.branch_id.0);
    assert!(change_summary.changed_record_count >= 1);
    assert!(change_summary.adjacency_delta_count <= change_summary.changed_record_count);
    assert!(patch_budget_summary.patch_record_count >= 1);
    assert_eq!(aspect_summary.changed_entity_aspect_count, 2);
    assert_eq!(aspect_summary.changed_relation_aspect_count, 0);
    assert_eq!(
        outcome.commit_log().structural_summary_event(),
        Some(structural_summary)
    );
    assert!(outcome
        .commit_log()
        .has_phase_started(CommitPhase::DraftPreparation));
    assert!(outcome
        .commit_log()
        .has_phase_completed(CommitPhase::Publication));
    assert_eq!(
        outcome.commit_log().history_summary_event(),
        Some(history_summary)
    );
    assert!(outcome.commit_log().events().iter().any(|event| matches!(
        event,
        CommitTraceEvent::InvariantEvaluated {
            execution_point: InvariantExecutionPoint::CommitBoundary,
            ..
        }
    )));
    assert!(outcome.commit_log().events().iter().any(|event| matches!(
        event,
        CommitTraceEvent::InvariantEvaluated {
            execution_point: InvariantExecutionPoint::MutationSensitive,
            ..
        }
    )));
    assert_eq!(
        outcome.commit_log().change_summary_event(),
        Some(change_summary)
    );
    assert_eq!(
        outcome.commit_log().patch_budget_summary_event(),
        Some(patch_budget_summary)
    );
    assert_eq!(
        outcome.commit_log().aspect_summary_event(),
        Some(aspect_summary)
    );
    assert!(outcome
        .commit_log()
        .events()
        .iter()
        .any(|event| matches!(event, CommitTraceEvent::DurableAppendPrepared { .. })));
    assert!(outcome.commit_log().has_commit_published());
    assert_eq!(
        outcome.commit_log().publication_summary_event(),
        Some(publication_summary)
    );
}

#[test]
fn commit_returns_envelope_with_patch_diagnostics_invariants_and_complexity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(execution, &runtime, "enveloped");
    let validation_summary = result.validation_summary();
    let structural_summary = result.structural_summary();
    let change_summary = result.change_summary().unwrap();
    let aspect_summary = result.aspect_summary().unwrap();
    let history_summary = result.history_summary().unwrap();
    let patch_budget_summary = result.patch_budget_summary().unwrap();
    let publication_summary = result.publication_summary().unwrap();

    assert!(!result.patch().is_empty());
    assert!(!result
        .envelope()
        .patch
        .authoritative_record_patches
        .is_empty());
    assert!(!result.diagnostics().is_empty());
    assert!(!structural_summary.invariant_groups.is_empty());
    assert!(!structural_summary.touched_partitions.is_empty());
    assert!(!result.invariant_executions().is_empty());
    assert!(result.invariant_executions().iter().any(|execution| {
        execution.metadata().execution_point() == InvariantExecutionPoint::CommitBoundary
    }));
    assert!(result.invariant_executions().iter().any(|execution| {
        execution.metadata().execution_point() == InvariantExecutionPoint::MutationSensitive
    }));
    assert!(result
        .invariant_executions()
        .iter()
        .all(|execution| execution.summary().result_count() >= execution.results().len()));
    assert_eq!(
        validation_summary.execution_count,
        result.invariant_executions().len()
    );
    assert!(validation_summary.executed_count >= 1);
    assert!(validation_summary.plan_backed_execution_count >= 1);
    assert!(validation_summary.commit_boundary_seen);
    assert!(validation_summary.mutation_sensitive_seen);
    assert!(validation_summary.committed_observation_count >= 1);
    assert!(validation_summary.speculative_observation_count >= 1);
    assert!(!validation_summary.consumed_groups.is_empty());
    assert!(!validation_summary.applicable_groups.is_empty());
    assert_eq!(
        validation_summary.result_count,
        result
            .invariant_executions()
            .iter()
            .map(|execution| execution.summary().result_count())
            .sum::<usize>()
    );
    assert_eq!(history_summary.parent_count, result.commit.parents.len());
    assert_eq!(
        change_summary.changed_record_count,
        result.changed_records.len()
    );
    assert_eq!(aspect_summary.changed_entity_aspect_count, 2);
    assert_eq!(aspect_summary.changed_relation_aspect_count, 0);
    assert_eq!(
        publication_summary.final_snapshot_id,
        Some(result.final_snapshot_id())
    );
    assert_eq!(
        publication_summary.patch_position,
        Some(result.patch_position())
    );
    assert_eq!(publication_summary.patch_record_count, result.patch().len());
    assert_eq!(
        patch_budget_summary.patch_record_count,
        result.patch().len()
    );
    assert!(result.complexity_delta().partitions_touched_by_commit >= 1);
    assert_eq!(
        result.outcome().commit.commit_id,
        result.envelope().commit.commit_id
    );
    assert_eq!(result.patch_position(), result.publication().patch_position);
    assert_eq!(result.final_snapshot_id(), result.snapshot.snapshot_id);
    assert_eq!(result.merge_parent_count(), 0);
}
