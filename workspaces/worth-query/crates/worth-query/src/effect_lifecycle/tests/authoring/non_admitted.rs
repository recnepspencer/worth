use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::{
    evaluate_effect_eligibility, normalize_raw_effect_intent, DeniedEffectEligibilityKind,
    EffectEligibilityOutcome, EffectFamily, RawEffectIntent,
};
use crate::workflow::{
    MergeLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
    WorkflowFreshnessPolicy, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;

use super::support::{
    durable_reload_effect_basis, preview_closeout_basis, preview_workflow_binding,
    runtime_workflow_binding, store_backed_effect_basis, tenant_mutation_basis, workflow_request,
};

#[test]
fn tenant_merge_denial_retains_normalized_effect_proof() {
    let normalized = normalize_raw_effect_intent(
        &tenant_mutation_basis().into(),
        RawEffectIntent::Merge {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MergeLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMerge,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MergeLoweringInput::reconcile_into_target(
                BranchId("main".to_string()),
                BranchId("candidate".to_string()),
            ),
        },
    )
    .expect("tenant merge should normalize");

    let denial =
        match evaluate_effect_eligibility(normalized.clone()).expect("fixture digest must fit") {
            EffectEligibilityOutcome::Denied(denial) => denial,
            other => panic!("expected denied effect, got {other:?}"),
        };

    assert_eq!(
        denial.denial_kind(),
        DeniedEffectEligibilityKind::BranchAuthorityRequired
    );
    assert_eq!(
        denial.normalized().normalized_digest(),
        normalized.normalized_digest()
    );
    assert_eq!(
        denial.normalized().basis_family(),
        BasisFamily::TenantScoped
    );
}

#[test]
fn preview_writeback_rebind_retains_normalized_effect_proof() {
    let normalized = normalize_raw_effect_intent(
        &preview_closeout_basis().into(),
        RawEffectIntent::Writeback {
            binding: preview_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("preview writeback should normalize");

    let rebind =
        match evaluate_effect_eligibility(normalized.clone()).expect("fixture digest must fit") {
            EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
            other => panic!("expected rebind-required effect, got {other:?}"),
        };

    assert_eq!(
        rebind.denial_kind(),
        DeniedEffectEligibilityKind::PreviewRebindRequired
    );
    assert_eq!(
        rebind.normalized().normalized_digest(),
        normalized.normalized_digest()
    );
    assert_eq!(rebind.normalized().basis_family(), BasisFamily::Preview);
}

#[test]
fn store_backed_writeback_returns_real_deferred_effect_proof() {
    let normalized = normalize_raw_effect_intent(
        &store_backed_effect_basis(),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("store-backed writeback should normalize");

    let deferred =
        match evaluate_effect_eligibility(normalized.clone()).expect("fixture digest must fit") {
            EffectEligibilityOutcome::Deferred(deferred) => deferred,
            other => panic!("expected deferred effect, got {other:?}"),
        };

    assert_eq!(
        deferred.denial_kind(),
        DeniedEffectEligibilityKind::StoreBackedExecutionDeferred
    );
    assert_eq!(
        deferred.normalized().normalized_digest(),
        normalized.normalized_digest()
    );
    assert_eq!(
        deferred.normalized().basis_family(),
        BasisFamily::StoreBacked
    );
    assert_eq!(deferred.normalized().family(), EffectFamily::Writeback);
    assert_eq!(deferred.counters().deferred_count(), 1);
}

#[test]
fn durable_reload_writeback_returns_real_deferred_effect_proof() {
    let normalized = normalize_raw_effect_intent(
        &durable_reload_effect_basis(),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("durable reload writeback should normalize");

    let deferred =
        match evaluate_effect_eligibility(normalized.clone()).expect("fixture digest must fit") {
            EffectEligibilityOutcome::Deferred(deferred) => deferred,
            other => panic!("expected deferred effect, got {other:?}"),
        };

    assert_eq!(
        deferred.denial_kind(),
        DeniedEffectEligibilityKind::DurableReplayDeferred
    );
    assert_eq!(
        deferred.normalized().normalized_digest(),
        normalized.normalized_digest()
    );
    assert_eq!(
        deferred.normalized().basis_family(),
        BasisFamily::DurableReload
    );
    assert_eq!(deferred.normalized().family(), EffectFamily::Writeback);
    assert_eq!(deferred.counters().deferred_count(), 1);
}
