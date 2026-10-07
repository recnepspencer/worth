use crate::basis_lifecycle::basis_lifecycle;
use crate::effect_lifecycle::{
    admit_effect_intent, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectDiagnosticsMaterialization, EffectDiagnosticsRequest,
    EffectEligibilityOutcome, EffectExecutionReceipt, EffectFamily,
    ExecutedEffectAuthorityArtifact, ExecutedEffectPlan, RawEffectIntent,
};
use crate::runtime::tests::support::{
    stateful_bridge_task_runtime, stateful_bridge_task_runtime_without_writeback,
    test_bridge_with_writeback_authority,
};
use crate::runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue};
use crate::workflow::{
    synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity,
    WorkflowAuthorityTargetFamily, WorkflowBindingScopeField, WorkflowBudgetClass,
    WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy, WritebackLoweringInput,
};

use super::super::{
    declare_writeback, projected_state_diff, writeback, WorthQueryWritebackDeclaration,
    WorthQueryWritebackStopSource,
};

#[test]
fn writeback_runs_the_real_effect_lifecycle_and_preserves_bridge_aftermath() {
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-workflow-writeback")
        .expect("workspace should open");
    let context = writeback(&workspace).expect("writeback authority should admit");
    let outcome = declare_writeback(projected_state_diff())
        .with_rich_inspection()
        .using(context)
        .run(&mut workspace);
    let completion = completed_or_panic(&outcome);

    assert_eq!(
        completion.receipt().declared_effect_family(),
        EffectFamily::Writeback
    );
    assert_eq!(
        completion.receipt().authority_owner(),
        crate::effect_lifecycle::EffectAuthorityOwner::WorthRuntimeBridge
    );
    assert_eq!(completion.counters().session_open_attempt_count(), 0);
    assert_eq!(
        completion
            .counters()
            .lower_runtime_execution_completed_count(),
        1
    );
    assert!(completion.diagnostics().is_some());
    assert!(!completion
        .aftermath()
        .outcome_identity_for_reporting()
        .is_empty());
    assert!(!completion
        .aftermath()
        .authority_receipt_identity_for_reporting()
        .is_empty());
    assert!(!completion
        .aftermath()
        .execution_receipt_identity_for_reporting()
        .is_empty());
}

#[test]
fn ordinary_and_explicit_writeback_paths_preserve_identical_evidence() {
    let declaration = declare_writeback(projected_state_diff()).with_rich_inspection();
    let explicit_workspace = stateful_bridge_task_runtime()
        .workspace("explicit-writeback-parity")
        .expect("explicit workspace should open");
    let (explicit_receipt, explicit_diagnostics) = explicit_writeback(
        &declaration,
        explicit_workspace
            .snapshot_identity()
            .expect("the fixture owner remains open"),
        true,
    );

    let mut ordinary_workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-writeback-parity")
        .expect("ordinary workspace should open");
    let context = writeback(&ordinary_workspace).expect("writeback authority should admit");
    let outcome = declaration.using(context).run(&mut ordinary_workspace);
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
fn operational_and_rich_writeback_differ_only_by_diagnostic_materialization() {
    let minimal = declare_writeback(projected_state_diff());
    let rich = minimal.clone().with_rich_inspection();
    let minimal = run_writeback("writeback-minimal", minimal);
    let rich = run_writeback("writeback-rich", rich);

    assert_eq!(
        minimal.admitted_effect().identity(),
        rich.admitted_effect().identity()
    );
    assert_eq!(minimal.lowered_plan(), rich.lowered_plan());
    assert_eq!(
        minimal.receipt().receipt_identity(),
        rich.receipt().receipt_identity()
    );
    assert_eq!(minimal.aftermath(), rich.aftermath());
    assert!(minimal.diagnostics().is_none());
    assert!(rich.diagnostics().is_some());
    assert_eq!(minimal.counters().inspection_materialization_count(), 0);
    assert_eq!(rich.counters().inspection_materialization_count(), 1);
}

#[test]
fn unsupported_foreign_and_stale_writeback_deny_before_lower_runtime_execution() {
    let unsupported = stateful_bridge_task_runtime_without_writeback()
        .workspace("unsupported-writeback")
        .expect("workspace should open");
    let unsupported = writeback(&unsupported).expect_err("missing authority must deny");
    assert_eq!(
        unsupported
            .counters()
            .lower_runtime_execution_attempt_count(),
        0
    );

    let left = stateful_bridge_task_runtime()
        .workspace("foreign-writeback-left")
        .expect("left should open");
    let foreign_context = writeback(&left).expect("left authority should admit");
    let mut right = stateful_bridge_task_runtime()
        .workspace("foreign-writeback-right")
        .expect("right should open");
    let foreign = declare_writeback(projected_state_diff())
        .using(foreign_context)
        .run(&mut right);
    let foreign = foreign.stop().expect("foreign authority must stop");
    assert_eq!(
        foreign.source(),
        WorthQueryWritebackStopSource::ForeignAuthority
    );
    assert_eq!(
        foreign.counters().lower_runtime_execution_attempt_count(),
        0
    );

    let stale_context = writeback(&right).expect("writeback authority should admit");
    right
        .insert("Task", |builder| {
            builder.set_aspect(
                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")
                    .expect("touch should admit"),
                WorthQueryAuthoredAspectValue::string("advance-writeback-basis"),
            )
        })
        .expect("mutation should advance the snapshot");
    let stale = declare_writeback(projected_state_diff())
        .using(stale_context)
        .run(&mut right);
    let stale = stale.stop().expect("stale authority must stop");
    assert_eq!(
        stale.source(),
        WorthQueryWritebackStopSource::StaleAuthority
    );
    assert_eq!(stale.counters().lower_runtime_execution_attempt_count(), 0);
}

fn run_writeback(
    workspace_name: &str,
    declaration: WorthQueryWritebackDeclaration,
) -> crate::ordinary::workflow::WorthQueryWritebackCompletion {
    let mut workspace = stateful_bridge_task_runtime()
        .workspace(workspace_name)
        .expect("workspace should open");
    let context = writeback(&workspace).expect("writeback authority should admit");
    match declaration.using(context).run(&mut workspace) {
        crate::ordinary::workflow::WorthQueryWritebackOutcome::Completed(completion) => completion,
        crate::ordinary::workflow::WorthQueryWritebackOutcome::Stopped(stop) => {
            panic!(
                "writeback stopped at {:?}: {}",
                stop.source(),
                stop.message()
            )
        }
    }
}

fn completed_or_panic(
    outcome: &crate::ordinary::workflow::WorthQueryWritebackOutcome,
) -> &crate::ordinary::workflow::WorthQueryWritebackCompletion {
    if let Some(stop) = outcome.stop() {
        panic!(
            "writeback stopped at {:?}: {}",
            stop.source(),
            stop.message()
        );
    }
    outcome.completed().expect("writeback should complete")
}

fn explicit_writeback(
    declaration: &WorthQueryWritebackDeclaration,
    snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    materialize_diagnostics: bool,
) -> (
    EffectExecutionReceipt,
    Option<EffectDiagnosticsMaterialization>,
) {
    let scope = WorkflowBindingScopeField::Identity(declaration.identity().evidence_identity());
    let binding = synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity(
        "ordinary-declarative-writeback",
        &scope,
        snapshot,
    );
    let basis = basis_lifecycle()
        .branch_head("main", true)
        .prepare_mutation()
        .expect("explicit basis should admit");
    let normalized = normalize_raw_effect_intent(
        &basis.into(),
        RawEffectIntent::Writeback {
            binding,
            request: WorkflowDeclarationRequest::new(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowCostClass::WritebackLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("explicit writeback should normalize");
    let eligibility = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => eligibility,
        other => panic!("explicit writeback should admit, got {other:?}"),
    };
    let lowered = scope_admitted_effect_plan(admit_effect_intent(eligibility))
        .lower()
        .expect("explicit writeback should lower");
    let bridge = test_bridge_with_writeback_authority();
    let execution = crate::effect_lifecycle::execute_lowered_writeback(
        &bridge,
        lowered
            .as_writeback()
            .expect("explicit lowering should preserve writeback"),
    )
    .expect("bridge writeback should execute");
    let receipt = ExecutedEffectPlan::new(
        lowered,
        ExecutedEffectAuthorityArtifact::Writeback { execution },
        1,
    )
    .receipt();
    let diagnostics = materialize_diagnostics
        .then(|| receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic()));
    (receipt, diagnostics)
}
