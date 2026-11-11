use crate::basis_lifecycle::BasisFamily;
use crate::workflow::{
    MergeLoweringInput, MutationLoweringInput, WorkflowAuthorityTargetFamily,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};

use crate::effect_lifecycle::{
    admit_effect_intent, discover_effect_lifecycle_support, evaluate_effect_eligibility,
    normalize_raw_effect_intent, DeniedEffectEligibilityKind, EffectAuthoringBasis,
    EffectEligibilityOutcome, EffectFamily, EffectIntentDenialKind, EffectSupportCause,
    RawEffectIntent,
};

use super::support::{
    branch_mutation_basis, native_name_patch, preview_closeout_basis, preview_workflow_binding,
    runtime_workflow_binding, tenant_mutation_basis, workflow_request,
};

#[test]
fn mutation_effect_normalizes_and_admits_from_raw_workflow_request() {
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(branch_mutation_basis()),
        RawEffectIntent::Mutation {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: EntityId::new(PartitionId(1), 7, 0),
                desired_aspect_fields: native_name_patch("esther"),
            },
        },
    )
    .expect("mutation effect should normalize");

    let admitted = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::Admitted(eligibility) => admit_effect_intent(eligibility),
        other => panic!("expected admitted effect, got {other:?}"),
    };

    assert_eq!(admitted.normalized().family(), EffectFamily::Mutation);
    assert_eq!(
        admitted.normalized().basis_family(),
        BasisFamily::BranchHead
    );
    assert_eq!(
        admitted
            .normalized()
            .workflow_request()
            .declaration_family(),
        &WorkflowDeclarationFamily::MutationLoweringNarrow
    );
    assert_eq!(
        admitted
            .workflow_declaration()
            .report()
            .authority_target_family(),
        &WorkflowAuthorityTargetFamily::RelationalMutation
    );
    assert!(!admitted.admitted_for_reporting().is_empty());
}

#[test]
fn preview_writeback_authoring_is_real_and_returns_typed_rebind() {
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(preview_closeout_basis()),
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

    let discovery =
        discover_effect_lifecycle_support(BasisFamily::Preview, EffectFamily::Writeback)
            .expect("fixture digest must fit");
    let rebind = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
        other => panic!("expected rebind-required effect, got {other:?}"),
    };

    assert_eq!(
        rebind.denial_kind(),
        DeniedEffectEligibilityKind::PreviewRebindRequired
    );
    assert_eq!(
        rebind.counters().support_lookup_width(),
        discovery.counters().support_lookup_width()
    );
    assert_eq!(
        rebind.counters().effect_support_row_count(),
        discovery.counters().support_lookup_width()
    );
}

#[test]
fn preview_mutation_authoring_is_real_and_returns_typed_rebind() {
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(preview_closeout_basis()),
        RawEffectIntent::Mutation {
            binding: preview_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: EntityId::new(PartitionId(1), 17, 0),
                desired_aspect_fields: native_name_patch("preview-mutation"),
            },
        },
    )
    .expect("preview mutation should normalize");

    let discovery = discover_effect_lifecycle_support(BasisFamily::Preview, EffectFamily::Mutation)
        .expect("fixture digest must fit");
    let rebind = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::RebindRequired(rebind) => rebind,
        other => panic!("expected rebind-required effect, got {other:?}"),
    };

    assert_eq!(
        rebind.denial_kind(),
        DeniedEffectEligibilityKind::PreviewRebindRequired
    );
    assert_eq!(
        rebind.decision_trace().message(),
        "preview-backed mutation must rebind to an authoritative basis before lowering"
    );
    assert_eq!(rebind.decision_trace().cause(), "preview_rebind_required");
    assert_eq!(
        rebind.counters().support_lookup_width(),
        discovery.counters().support_lookup_width()
    );
    assert_eq!(
        rebind.counters().effect_support_row_count(),
        discovery.counters().support_lookup_width()
    );
}

#[test]
fn tenant_scoped_merge_denies_with_same_support_width_as_discovery() {
    let normalized = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(tenant_mutation_basis()),
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
    .expect("merge effect should normalize");

    let discovery =
        discover_effect_lifecycle_support(BasisFamily::TenantScoped, EffectFamily::Merge)
            .expect("fixture digest must fit");
    let denial = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::Denied(denial) => denial,
        other => panic!("expected denied effect, got {other:?}"),
    };

    assert_eq!(
        denial.denial_kind(),
        DeniedEffectEligibilityKind::BranchAuthorityRequired
    );
    assert_eq!(denial.counters().denied_count(), 1);
    assert_eq!(denial.decision_trace().cause(), "branch_authority_required");
    assert_eq!(
        denial.counters().support_lookup_width(),
        discovery.counters().support_lookup_width()
    );
    assert_eq!(
        discovery.cause(),
        EffectSupportCause::BranchAuthorityRequired
    );
}

#[test]
fn normalization_rejects_workflow_target_mismatch() {
    let denial = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(branch_mutation_basis()),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect_err("mismatched workflow target should deny normalization");

    assert_eq!(
        denial.denial_kind(),
        EffectIntentDenialKind::WorkflowAuthorityTargetMismatch
    );
    assert_eq!(denial.counters().workflow_authority_target_check_count(), 1);
}

#[test]
fn normalization_rejects_preview_basis_with_runtime_workflow_binding() {
    let denial = normalize_raw_effect_intent(
        &EffectAuthoringBasis::from(preview_closeout_basis()),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect_err("preview basis must reject runtime-only workflow binding");

    assert_eq!(
        denial.denial_kind(),
        EffectIntentDenialKind::BasisWorkflowBindingMismatch
    );
}
