use worth_relational::facade::history::BranchId;

use crate::basis_lifecycle::basis_lifecycle;
use crate::effect_lifecycle::{
    admit_effect_intent, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectDiagnosticsMaterialization, EffectDiagnosticsRequest,
    EffectEligibilityOutcome, EffectExecutionAuthority, EffectExecutionReceipt, EffectFamily,
    RawEffectIntent,
};
use crate::harness::fixtures::effect_authorities::{
    branch_snapshot_identity, create_entity, relational_runtime_with_intent_strategy,
};
use crate::runtime::tests::support::{
    stateful_bridge_task_runtime, stateful_bridge_task_runtime_with_merge,
};
use crate::workflow::{
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity,
    MergeLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBindingScopeField,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy,
};

use super::super::{
    branch_merge, declare_branch_merge, WorthQueryBranchMergeCompletion,
    WorthQueryBranchMergeDeclaration, WorthQueryBranchMergeDeclarationDenialKind,
    WorthQueryBranchMergeStopSource,
};

#[test]
fn branch_merge_runs_the_real_relational_effect_lifecycle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = merge_declaration().with_rich_inspection();
    let mut workspace = stateful_bridge_task_runtime_with_merge(request_execution)
        .workspace("ordinary-branch-merge")
        .expect("workspace should open");
    let context = branch_merge(request_execution, &workspace, &declaration)
        .expect("merge authority should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    let completion = completed_or_panic(&outcome);

    assert_eq!(
        completion.receipt().declared_effect_family(),
        EffectFamily::Merge
    );
    assert_eq!(
        completion.receipt().authority_owner(),
        crate::effect_lifecycle::EffectAuthorityOwner::WorthRelational
    );
    assert_eq!(completion.counters().session_open_attempt_count(), 0);
    assert_eq!(
        completion
            .counters()
            .lower_runtime_execution_completed_count(),
        1
    );
    assert_eq!(completion.receipt().write_count(), 1);
    assert!(completion.aftermath().commit_id() > 0);
    assert!(completion.aftermath().version_id() > 0);
    assert!(completion.diagnostics().is_some());
}

#[test]
fn ordinary_and_explicit_branch_merge_preserve_identical_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = merge_declaration().with_rich_inspection();
    let (explicit_receipt, explicit_diagnostics) =
        explicit_merge(request_execution, &declaration, true);

    let mut workspace = stateful_bridge_task_runtime_with_merge(request_execution)
        .workspace("ordinary-branch-merge-parity")
        .expect("workspace should open");
    let context = branch_merge(request_execution, &workspace, &declaration)
        .expect("merge authority should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    let ordinary = completed_or_panic(&outcome);

    assert_eq!(
        ordinary.admitted_effect().identity(),
        explicit_receipt
            .decision_trace()
            .admitted_or_batch_identity()
    );
    assert_eq!(
        ordinary.lowered_plan().request_identity(),
        explicit_receipt.decision_trace().lowered_identity()
    );
    assert_eq!(
        ordinary.receipt().receipt_identity(),
        explicit_receipt.receipt_identity()
    );
    assert_eq!(
        ordinary.receipt().target_evidence(),
        explicit_receipt.target_evidence()
    );
    assert_eq!(
        ordinary.aftermath().identity(),
        explicit_receipt
            .integrity_markers()
            .authority_artifact_identity()
    );
    assert_eq!(
        ordinary
            .diagnostics()
            .expect("ordinary diagnostics should materialize")
            .diagnostics_identity(),
        explicit_diagnostics
            .expect("explicit diagnostics should materialize")
            .diagnostics_identity()
    );
}

#[test]
fn merge_diagnostic_policy_changes_only_derived_inspection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let operational = run_merge(
        request_execution,
        "branch-merge-operational",
        merge_declaration(),
    );
    let rich = run_merge(
        request_execution,
        "branch-merge-rich",
        merge_declaration().with_rich_inspection(),
    );

    assert_eq!(operational.admitted_effect(), rich.admitted_effect());
    assert_eq!(operational.lowered_plan(), rich.lowered_plan());
    assert_eq!(
        operational.receipt().receipt_identity(),
        rich.receipt().receipt_identity()
    );
    assert_eq!(operational.aftermath(), rich.aftermath());
    assert!(operational.diagnostics().is_none());
    assert!(rich.diagnostics().is_some());
    assert_eq!(operational.counters().inspection_materialization_count(), 0);
    assert_eq!(rich.counters().inspection_materialization_count(), 1);
}

#[path = "branch_merge/invalid_unsupported_foreign_mismatched_and_stale_merges_do_no_lo.rs"]
mod invalid_unsupported_foreign_mismatched_and_stale_merges_do_no_lo;

fn merge_declaration() -> WorthQueryBranchMergeDeclaration {
    declare_branch_merge("main", "candidate").expect("merge should declare")
}

fn run_merge(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    declaration: WorthQueryBranchMergeDeclaration,
) -> WorthQueryBranchMergeCompletion {
    let mut workspace = stateful_bridge_task_runtime_with_merge(request_execution)
        .workspace(name)
        .expect("workspace should open");
    let context = branch_merge(request_execution, &workspace, &declaration)
        .expect("merge authority should admit");
    match declaration
        .using(context)
        .run(request_execution, &mut workspace)
    {
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::Completed(completion) => {
            completion
        }
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::Stopped(stop) => {
            panic!(
                "branch merge stopped at {:?}: {}",
                stop.source(),
                stop.message()
            )
        }
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::Deferred(deferred) => {
            panic!("branch merge deferred: {}", deferred.message())
        }
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::ControlStopped(stopped) => {
            panic!("branch merge control stopped: {}", stopped.message())
        }
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::DurabilityUnresolved(
            unresolved,
        ) => {
            panic!(
                "branch merge durability unresolved: {}",
                unresolved.message()
            )
        }
    }
}

fn completed_or_panic(
    outcome: &crate::ordinary::workflow::WorthQueryBranchMergeOutcome,
) -> &WorthQueryBranchMergeCompletion {
    if let Some(stop) = outcome.stop() {
        panic!(
            "branch merge stopped at {:?}: {}",
            stop.source(),
            stop.message()
        );
    }
    outcome.completed().expect("branch merge should complete")
}

fn explicit_merge(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    declaration: &WorthQueryBranchMergeDeclaration,
    materialize_diagnostics: bool,
) -> (
    EffectExecutionReceipt,
    Option<EffectDiagnosticsMaterialization>,
) {
    let mut runtime = relational_runtime_with_intent_strategy();
    create_entity(
        request_execution,
        &mut runtime,
        "main",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        request_execution,
        &mut runtime,
        BranchId("candidate".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("candidate branch should be created");
    create_entity(
        request_execution,
        &mut runtime,
        "candidate-only",
        BranchId("candidate".to_string()),
    );
    let scope = WorkflowBindingScopeField::Identity(declaration.identity().evidence_identity());
    let binding = synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity(
        "ordinary-declarative-branch-merge",
        &scope,
        branch_snapshot_identity(&runtime, "main"),
        BranchId("main".to_string()),
    );
    let basis = basis_lifecycle()
        .branch_head("main", true)
        .prepare_mutation()
        .expect("explicit merge basis should admit");
    let normalized = normalize_raw_effect_intent(
        &basis.into(),
        RawEffectIntent::Merge {
            binding,
            request: WorkflowDeclarationRequest::new(
                WorkflowDeclarationFamily::MergeLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMerge,
                WorkflowCostClass::MergeLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MergeLoweringInput::reconcile_into_target(
                BranchId("main".to_string()),
                BranchId("candidate".to_string()),
            ),
        },
    )
    .expect("explicit merge should normalize");
    let eligibility = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => eligibility,
        other => panic!("explicit merge should admit, got {other:?}"),
    };
    let lowered = scope_admitted_effect_plan(admit_effect_intent(eligibility))
        .lower()
        .expect("explicit merge should lower");
    let receipt = lowered
        .execute_with(
            request_execution,
            EffectExecutionAuthority::crate_relational_without_application_schema(&mut runtime),
        )
        .expect("explicit merge should execute")
        .receipt();
    let diagnostics = materialize_diagnostics
        .then(|| receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic()));
    (receipt, diagnostics)
}
