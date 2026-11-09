use crate::workflow::{
    MergeLoweringInput, MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBasisFamily,
    WorkflowContextBinding, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WritebackLoweringInput,
};

use super::authoring_basis::EffectAuthoringBasis;
use super::normalized::{EffectIntentDenial, EffectOperationInput, NormalizedEffectIntent};
use super::taxonomy::{EffectAuthorityLane, EffectFamily, EffectIntentDenialKind};

#[derive(Clone, Debug, PartialEq)]
pub enum RawEffectIntent {
    Mutation {
        binding: WorkflowContextBinding,
        request: WorkflowDeclarationRequest,
        input: MutationLoweringInput,
    },
    Merge {
        binding: WorkflowContextBinding,
        request: WorkflowDeclarationRequest,
        input: MergeLoweringInput,
    },
    Writeback {
        binding: WorkflowContextBinding,
        request: WorkflowDeclarationRequest,
        input: WritebackLoweringInput,
    },
}

pub fn normalize_raw_effect_intent(
    authoring_basis: &EffectAuthoringBasis,
    raw: RawEffectIntent,
) -> Result<NormalizedEffectIntent, EffectIntentDenial> {
    match raw {
        RawEffectIntent::Mutation {
            binding,
            request,
            input,
        } => normalize(
            authoring_basis,
            binding,
            request,
            WorkflowDeclarationFamily::MutationLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMutation,
            EffectFamily::Mutation,
            EffectAuthorityLane::Relational,
            EffectOperationInput::Mutation(input),
            "raw_effect.mutation",
        ),
        RawEffectIntent::Merge {
            binding,
            request,
            input,
        } => normalize(
            authoring_basis,
            binding,
            request,
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            EffectFamily::Merge,
            EffectAuthorityLane::Relational,
            EffectOperationInput::Merge(input),
            "raw_effect.merge",
        ),
        RawEffectIntent::Writeback {
            binding,
            request,
            input,
        } => normalize(
            authoring_basis,
            binding,
            request,
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
            EffectFamily::Writeback,
            EffectAuthorityLane::RuntimeBridge,
            EffectOperationInput::Writeback(input),
            "raw_effect.writeback",
        ),
    }
}

fn normalize(
    authoring_basis: &EffectAuthoringBasis,
    binding: WorkflowContextBinding,
    request: WorkflowDeclarationRequest,
    expected_family: WorkflowDeclarationFamily,
    expected_target: WorkflowAuthorityTargetFamily,
    effect_family: EffectFamily,
    authority_lane: EffectAuthorityLane,
    operation_input: EffectOperationInput,
    source_path: &'static str,
) -> Result<NormalizedEffectIntent, EffectIntentDenial> {
    if request.declaration_family() != &expected_family {
        return Err(EffectIntentDenial::new(
            EffectIntentDenialKind::WorkflowDeclarationFamilyMismatch,
            "workflow declaration family does not match the requested effect family",
            source_path,
            1,
            basis_pairing_check_width(authoring_basis),
        ));
    }

    if request.authority_target_family() != &expected_target {
        return Err(EffectIntentDenial::new(
            EffectIntentDenialKind::WorkflowAuthorityTargetMismatch,
            "workflow authority target family does not match the requested effect lane",
            source_path,
            1,
            basis_pairing_check_width(authoring_basis),
        ));
    }

    if !binding_matches_basis(authoring_basis, binding.basis_family()) {
        return Err(EffectIntentDenial::new(
            EffectIntentDenialKind::BasisWorkflowBindingMismatch,
            "effect authoring basis and workflow binding family must stay in the same authority story",
            source_path,
            1,
            basis_pairing_check_width(authoring_basis),
        ));
    }

    Ok(NormalizedEffectIntent::new(
        authoring_basis,
        effect_family,
        authority_lane,
        binding,
        request,
        operation_input,
        source_path,
    )
    .map_err(|cause| {
        EffectIntentDenial::new(
            EffectIntentDenialKind::EvidenceIdentityDerivationDenied(cause),
            "normalized effect identity exceeded its digest bounds",
            source_path,
            1,
            basis_pairing_check_width(authoring_basis),
        )
    })?)
}

fn binding_matches_basis(
    authoring_basis: &EffectAuthoringBasis,
    binding_family: &WorkflowBasisFamily,
) -> bool {
    match authoring_basis.requires_preview_workflow_binding() {
        true => matches!(
            binding_family,
            WorkflowBasisFamily::PreviewFoundation
                | WorkflowBasisFamily::PreviewPromotionComparison
        ),
        false => binding_family == &WorkflowBasisFamily::RuntimePreflight,
    }
}

fn basis_pairing_check_width(authoring_basis: &EffectAuthoringBasis) -> usize {
    usize::from(authoring_basis.requires_preview_workflow_binding())
        + usize::from(!authoring_basis.requires_preview_workflow_binding())
}
