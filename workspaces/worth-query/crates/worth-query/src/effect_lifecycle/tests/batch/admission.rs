use crate::effect_lifecycle::{
    admit_effect_batch_components, effect_batch, DeniedEffectEligibilityKind, EffectAuthoringBasis,
    EffectBatchAdmissionDenialKind, EffectSupportCause,
};
use worth_relational::facade::identity::PartitionId;

use super::super::support::{
    admitted_alternate_branch_mutation_effect, admitted_branch_merge_effect,
    admitted_mutation_effect, admitted_tenant_mutation_effect, admitted_tenant_writeback_effect,
    native_name_patch, preview_closeout_basis, preview_derived_inspection_advisory,
    preview_workflow_binding, runtime_workflow_binding, store_backed_effect_basis,
    workflow_request,
};
use crate::workflow::{
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily,
    WorkflowFreshnessPolicy,
};

#[test]
fn batch_admission_denies_mixed_authority_components() {
    let denial = admit_effect_batch_components(vec![
        admitted_mutation_effect(),
        admitted_tenant_writeback_effect(),
    ])
    .expect_err("mixed authority components should deny");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::MixedAuthorityLane
    );
}

#[test]
fn batch_admission_denies_mixed_basis_components() {
    let denial = admit_effect_batch_components(vec![
        admitted_mutation_effect(),
        admitted_tenant_mutation_effect(),
    ])
    .expect_err("mixed basis components should deny");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::MixedBasisLane
    );
}

#[test]
fn batch_admission_denies_distinct_basis_identity_within_one_basis_lane() {
    let denial = admit_effect_batch_components(vec![
        admitted_mutation_effect(),
        admitted_alternate_branch_mutation_effect(),
    ])
    .expect_err("distinct branch-head bases should deny even within one basis family");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::MixedBasisIdentity
    );
}

#[test]
fn batch_admission_denies_unsupported_non_mutation_family() {
    let denial = admit_effect_batch_components(vec![admitted_branch_merge_effect()])
        .expect_err("merge-only batch should deny until batch family support is explicit");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::UnsupportedBatchFamily(
            crate::effect_lifecycle::EffectFamily::Merge,
        )
    );
}

#[test]
fn batch_admission_denies_preview_mutation_until_authoritative_rebind() {
    let denial = effect_batch()
        .using_basis(EffectAuthoringBasis::from(preview_closeout_basis()))
        .push(crate::effect_lifecycle::RawEffectIntent::Mutation {
            binding: preview_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: worth_relational::facade::identity::EntityId::new(PartitionId(1), 33, 0),
                desired_aspect_fields: native_name_patch("preview-batch-mutation"),
            },
        })
        .admit()
        .expect_err("preview mutation batch should surface explicit rebind before admission");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::ComponentRebindRequired(
            crate::effect_lifecycle::DeniedEffectEligibilityKind::PreviewRebindRequired,
        )
    );
}

#[test]
fn batch_admission_denies_advisory_preview_derived_mutation_components() {
    let denial = effect_batch()
        .using_basis(EffectAuthoringBasis::from(
            preview_derived_inspection_advisory(),
        ))
        .push(crate::effect_lifecycle::RawEffectIntent::Mutation {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::MutationLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMutation,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MutationLoweringInput::IntentReconciliation {
                entity_id: worth_relational::facade::identity::EntityId::new(PartitionId(1), 34, 0),
                desired_aspect_fields: native_name_patch("advisory-batch-mutation"),
            },
        })
        .admit()
        .expect_err("advisory components must not batch-admit into executable effects");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::ComponentAdvisory(
            EffectSupportCause::AdvisoryOnlyExecution,
        )
    );
}

#[test]
fn batch_admission_preserves_exact_deferred_denial_kind() {
    let denial = effect_batch()
        .using_basis(store_backed_effect_basis())
        .push(crate::effect_lifecycle::RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: crate::workflow::WritebackLoweringInput::projected_state_diff(),
        })
        .admit()
        .expect_err("store-backed batch should preserve exact deferred cause");

    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::ComponentDeferred(
            DeniedEffectEligibilityKind::StoreBackedExecutionDeferred,
        )
    );
    assert_eq!(
        denial
            .deferred_contract()
            .expect("deferred batch denial should preserve exact deferred contract")
            .neighbor_family(),
        crate::effect_lifecycle::EffectDeferredNeighborFamily::StoreBackedExecutionParity
    );
    assert!(denial
        .deferred_contract()
        .expect("deferred batch denial should preserve exact deferred contract")
        .leaves_zero_operational_residue());
}

#[test]
fn a_mutation_first_batch_still_refuses_a_later_merge_component() {
    let denial = admit_effect_batch_components(vec![
        admitted_mutation_effect(),
        admitted_branch_merge_effect(),
    ])
    .expect_err("the first mutation cannot authorize a later merge component");
    assert_eq!(
        denial.denial_kind(),
        &EffectBatchAdmissionDenialKind::UnsupportedBatchFamily(
            crate::effect_lifecycle::EffectFamily::Merge
        )
    );
    assert_eq!(denial.component_index(), Some(1));
}
