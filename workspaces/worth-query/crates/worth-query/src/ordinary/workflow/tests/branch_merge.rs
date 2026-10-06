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
    stateful_bridge_task_runtime_with_merge_durable_fault,
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
    WorthQueryBranchMergeNextAction, WorthQueryBranchMergeStopSource,
};

#[test]
fn branch_merge_runs_the_real_relational_effect_lifecycle() {
    let declaration = merge_declaration().with_rich_inspection();
    let mut workspace = stateful_bridge_task_runtime_with_merge()
        .workspace("ordinary-branch-merge")
        .expect("workspace should open");
    let context = branch_merge(&workspace, &declaration).expect("merge authority should admit");
    let outcome = declaration.using(context).run(&mut workspace);
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
fn ordinary_branch_merge_preserves_settlement_from_its_real_backend_path() {
    let declaration = merge_declaration();
    let (runtime, probe) = stateful_bridge_task_runtime_with_merge_durable_fault();
    let mut workspace = runtime
        .workspace("ordinary-branch-merge-durable-fault")
        .expect("workspace should open");
    let context = branch_merge(&workspace, &declaration).expect("merge authority should admit");

    let outcome = declaration.using(context).run(&mut workspace);
    let deferred = outcome
        .settlement_deferred()
        .expect("performed merge must return a settlement-deferred outcome");
    assert_eq!(
        deferred.next_action(),
        WorthQueryBranchMergeNextAction::RepairDeferredBranchMergeSettlement
    );
    let commit_id = deferred.commit_id();
    assert_eq!(
        deferred.counters().lower_runtime_execution_attempt_count(),
        1
    );
    assert_eq!(
        deferred
            .counters()
            .lower_runtime_execution_completed_count(),
        1
    );
    assert_eq!(deferred.counters().settlement_deferred_count(), 1);

    let mut foreign_workspace = stateful_bridge_task_runtime_with_merge()
        .workspace("ordinary-branch-merge-foreign-settlement")
        .expect("foreign workspace opens");
    assert!(matches!(
        foreign_workspace.repair_deferred_branch_merge_settlement(deferred),
        Err(crate::runtime::WorthQuerySettlementRepairError::Settlement(
            worth_relational::facade::publication::DeferredPublicationSettlementError::ForeignRuntime {
                ..
            }
        ))
    ));
    drop(outcome);

    let repaired = workspace
        .repair_pending_branch_merge_settlement(commit_id)
        .expect("public workspace owner repairs settlement after the token is dropped");
    let repeated = workspace
        .repair_pending_branch_merge_settlement(commit_id)
        .expect("public workspace settlement repair is idempotent");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(repeated, repaired);
    assert_eq!(
        probe.main_entity_count(),
        2,
        "settlement recovery must preserve the source-only entity on the target branch"
    );

    let child = declare_branch_merge("candidate", "main").expect("child merge declares");
    let child_context = branch_merge(&workspace, &child).expect("child merge authority admits");
    let child_outcome = child.using(child_context).run(&mut workspace);
    assert!(
        child_outcome.completed().is_some(),
        "a subsequent public operation must proceed after repair"
    );
}

#[test]
fn ordinary_and_explicit_branch_merge_preserve_identical_evidence() {
    let declaration = merge_declaration().with_rich_inspection();
    let (explicit_receipt, explicit_diagnostics) = explicit_merge(&declaration, true);

    let mut workspace = stateful_bridge_task_runtime_with_merge()
        .workspace("ordinary-branch-merge-parity")
        .expect("workspace should open");
    let context = branch_merge(&workspace, &declaration).expect("merge authority should admit");
    let outcome = declaration.using(context).run(&mut workspace);
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
    let operational = run_merge("branch-merge-operational", merge_declaration());
    let rich = run_merge(
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

#[test]
fn invalid_unsupported_foreign_mismatched_and_stale_merges_do_no_lower_work() {
    let invalid = declare_branch_merge("main", "main").expect_err("same branch must deny");
    assert_eq!(
        invalid.denial_kind(),
        WorthQueryBranchMergeDeclarationDenialKind::SameBranch
    );

    let unsupported = stateful_bridge_task_runtime()
        .workspace("unsupported-branch-merge")
        .expect("workspace should open");
    let declaration = merge_declaration();
    let unsupported = branch_merge(&unsupported, &declaration)
        .expect_err("missing relational authority must deny");
    assert_eq!(
        unsupported
            .counters()
            .lower_runtime_execution_attempt_count(),
        0
    );

    let left = stateful_bridge_task_runtime_with_merge()
        .workspace("foreign-branch-merge-left")
        .expect("left should open");
    let foreign_context = branch_merge(&left, &declaration).expect("left should admit");
    let mut right = stateful_bridge_task_runtime_with_merge()
        .workspace("foreign-branch-merge-right")
        .expect("right should open");
    let foreign = declaration.clone().using(foreign_context).run(&mut right);
    let foreign = foreign.stop().expect("foreign authority must stop");
    assert_eq!(
        foreign.source(),
        WorthQueryBranchMergeStopSource::ForeignAuthority
    );
    assert_eq!(
        foreign.counters().lower_runtime_execution_attempt_count(),
        0
    );

    let captured = branch_merge(&right, &declaration).expect("context should admit");
    let reverse = declare_branch_merge("candidate", "main").expect("reverse should declare");
    let mismatch = reverse.using(captured).run(&mut right);
    let mismatch = mismatch.stop().expect("mismatched context must stop");
    assert_eq!(
        mismatch.source(),
        WorthQueryBranchMergeStopSource::MismatchedContext
    );
    assert_eq!(
        mismatch.counters().lower_runtime_execution_attempt_count(),
        0
    );

    let first = branch_merge(&right, &declaration).expect("first context should admit");
    let stale = branch_merge(&right, &declaration).expect("second context should admit");
    let first = declaration.clone().using(first).run(&mut right);
    assert!(first.completed().is_some(), "first merge should execute");
    let stale = declaration.using(stale).run(&mut right);
    let stale = stale.stop().expect("retained context must stale-deny");
    assert_eq!(
        stale.source(),
        WorthQueryBranchMergeStopSource::StaleAuthority
    );
    assert_eq!(stale.counters().lower_runtime_execution_attempt_count(), 0);
}

fn merge_declaration() -> WorthQueryBranchMergeDeclaration {
    declare_branch_merge("main", "candidate").expect("merge should declare")
}

fn run_merge(
    name: &str,
    declaration: WorthQueryBranchMergeDeclaration,
) -> WorthQueryBranchMergeCompletion {
    let mut workspace = stateful_bridge_task_runtime_with_merge()
        .workspace(name)
        .expect("workspace should open");
    let context = branch_merge(&workspace, &declaration).expect("merge authority should admit");
    match declaration.using(context).run(&mut workspace) {
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
        crate::ordinary::workflow::WorthQueryBranchMergeOutcome::SettlementDeferred(deferred) => {
            panic!("branch merge requires settlement: {}", deferred.message())
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
    declaration: &WorthQueryBranchMergeDeclaration,
    materialize_diagnostics: bool,
) -> (
    EffectExecutionReceipt,
    Option<EffectDiagnosticsMaterialization>,
) {
    let mut runtime = relational_runtime_with_intent_strategy();
    create_entity(&mut runtime, "main", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("candidate".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("candidate branch should be created");
    create_entity(
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
            EffectExecutionAuthority::crate_relational_without_application_schema(&mut runtime),
        )
        .expect("explicit merge should execute")
        .receipt();
    let diagnostics = materialize_diagnostics
        .then(|| receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic()));
    (receipt, diagnostics)
}
