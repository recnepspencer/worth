use std::sync::Arc;

use crate::diagnostics::data::{DiagnosticsArtifactKind, RelationalDiagnosticValue};
use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::history::BranchId;
use crate::facade::merge::{
    DeletionExecutionClass, MergeConflictClass, MergeExecutableClass, MergeExecutionRequest,
    MergeIntent, MergeResolutionClass,
};
use crate::facade::transactions::TransactionId;
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch, diagnostic_field,
    diagnostic_field_optional, durable_runtime_for_medium, OwnedTestDurableMedium,
};

fn prepared_merge_promoted_to_deleted_on_both_sides(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::facade::runtime::RelationalRuntime,
) -> crate::facade::merge::PreparedMergeExecution {
    create_entity(execution, &*runtime, "root");
    create_branch_from_main(runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let mut prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge");

    {
        let execution_ready = prepared.execution_ready_plan_mut_for_test();
        let lowered = Arc::make_mut(&mut execution_ready.lowered_records);
        lowered[0].classification = MergeConflictClass::Deletion(
            crate::merge::data::DeletionMergeClass::DeletedOnBothSides,
        );
        lowered[0].resolution_class =
            MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides);
        lowered[0].executable_class = Some(MergeExecutableClass::ConvergeDeletedOnBothSides);
        lowered[0].policy_proof_boundary = crate::merge::data::MergePolicyProofBoundary {
            ownership_surface: crate::facade::merge::MergePolicyOwnershipSurface::RuntimeOnly,
            decision_boundary: crate::merge::data::MergePolicyDecisionBoundary::AutoResolved,
        };
        lowered[0].readiness = crate::facade::merge::MergeExecutionReadiness::Admitted;
        lowered[0].record_decision = crate::facade::merge::LoweredRecordDecision::Execute(
            crate::merge::data::LoweredRecordExecutionBundle {
                kind:
                    crate::merge::data::LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides,
                aspects: Arc::from([]),
            },
        );
        lowered[0].lowered_action =
            Some(crate::facade::merge::LoweredMergeAction::ConvergeDeletedOnBothSides);
        lowered[0].blocked_reason = None;
        lowered[0].rejected_reason = None;
        lowered[0].aspect_outcomes = Arc::from([]);
    }

    let compiled = runtime
        .merge()
        .compile_execution_ready_merge_plan_for_test(prepared.execution_ready_plan_mut_for_test())
        .expect("compiled promoted executable plan");
    runtime
        .merge()
        .replace_bound_merge_plan_for_test(execution, &mut prepared, compiled)
        .expect("promoted mutation plan");
    prepared
}

#[test]
fn promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let mut runtime = durable_runtime_for_medium(&medium);
    let prepared = prepared_merge_promoted_to_deleted_on_both_sides(execution, &mut runtime);

    let plan = prepared.bound_executable_plan();
    assert_eq!(plan.record_plans.len(), 1);
    match &plan.record_plans[0] {
        crate::merge::data::BoundExecutableMergeRecordPlan::ConvergeDeletedOnBothSides(plan) => {
            assert_eq!(
                plan.provenance.resolution_class,
                MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides)
            );
            assert_eq!(
                plan.provenance.executable_class,
                MergeExecutableClass::ConvergeDeletedOnBothSides
            );
            assert_eq!(
                plan.semantics,
                crate::merge::data::DeletedOnBothSidesSemantics::AuthoritativeMutualDeletionConvergence
            );
            assert_eq!(
                plan.lineage_continuity,
                crate::merge::data::MergeLineageContinuityVerdict::Unchanged
            );
        }
        other => panic!("expected deleted-on-both-sides record plan, got {other:?}"),
    }
}

#[test]
fn promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let mut runtime = durable_runtime_for_medium(&medium);
    let prepared = prepared_merge_promoted_to_deleted_on_both_sides(execution, &mut runtime);

    let plan = prepared.bind_mutation_plan_for_test(TransactionId(701));

    assert_eq!(plan.structural_summary.executed_record_count, 1);
    assert_eq!(
        plan.structural_summary
            .converged_deleted_on_both_sides_count,
        1
    );
    assert_eq!(
        plan.structural_summary
            .deleted_on_both_sides_lineage_unchanged_count,
        1
    );
    assert_eq!(plan.structural_summary.emitted_mutation_intent_count, 0);
    assert!(plan.merged_plan.merged_intents.is_empty());
    assert_eq!(
        plan.merge_execution_summary
            .converged_deleted_on_both_sides_count,
        1
    );
    assert_eq!(
        plan.merge_execution_summary
            .deleted_on_both_sides_lineage_unchanged_count,
        1
    );
}

#[test]
fn promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let mut runtime = durable_runtime_for_medium(&medium);
    let prepared = prepared_merge_promoted_to_deleted_on_both_sides(execution, &mut runtime);

    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("canonical envelope");

    assert_eq!(merge.structural_summary.executed_record_count, 1);
    assert_eq!(
        merge
            .structural_summary
            .converged_deleted_on_both_sides_count,
        1
    );
    assert_eq!(
        merge
            .structural_summary
            .deleted_on_both_sides_lineage_unchanged_count,
        1
    );
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0);
    assert!(envelope.merged_plan.merged_intents.is_empty());

    let summary_entry = envelope
        .diagnostics_summary
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
        .expect("merge execution summary entry");
    assert_eq!(
        diagnostic_field(summary_entry, "converged_deleted_on_both_sides_count"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
    assert_eq!(
        diagnostic_field(
            summary_entry,
            "deleted_on_both_sides_lineage_unchanged_count"
        ),
        &RelationalDiagnosticValue::Unsigned(1)
    );

    let diagnostics = runtime.publication().diagnostics();
    let success_artifact = diagnostics
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.kind == DiagnosticsArtifactKind::DetailedTrace
                && artifact.entries.iter().any(|entry| {
                    entry.code == DiagnosticCode::MergeExecutionPublished
                        && diagnostic_field(entry, "commit_id")
                            == &RelationalDiagnosticValue::CommitId(merge.commit.commit.commit_id)
                })
        })
        .expect("merge execution success artifact");
    let record_entry = success_artifact
        .entries
        .iter()
        .find(|entry| {
            diagnostic_field_optional(entry, "record_class")
                == Some(&RelationalDiagnosticValue::String(
                    "converge_deleted_on_both_sides".to_string(),
                ))
        })
        .expect("deleted-on-both-sides record entry");
    assert!(matches!(
        diagnostic_field(record_entry, "equality_witness_digest"),
        RelationalDiagnosticValue::String(_)
    ));
    assert_eq!(
        diagnostic_field(record_entry, "deletion_semantics"),
        &RelationalDiagnosticValue::String("AuthoritativeMutualDeletionConvergence".to_string())
    );
    assert_eq!(
        diagnostic_field(record_entry, "lineage_continuity"),
        &RelationalDiagnosticValue::String("Unchanged".to_string())
    );
}
