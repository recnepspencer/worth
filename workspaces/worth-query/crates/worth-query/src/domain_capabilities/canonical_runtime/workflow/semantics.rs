use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPayload, WorthQueryWorkflowContributionPosture,
    WorthQueryWorkflowRuntimeBindingSemantics, WorthQueryWorkflowRuntimeSemantics,
};
#[cfg(test)]
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowInspectionSemantics, WorthQueryWorkflowLoweringSemantics,
};
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::workflow::WorkflowPreviewEvaluationClass;

pub(super) fn workflow_source_label<T>(
    target: &T,
    payload: &WorthQueryWorkflowContributionPayload,
) -> String
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    if let Some((name, ..)) = target.semantics().intent_declaration() {
        return name.to_string();
    }
    if let Some((family, entrypoint, ..)) = target.semantics().admitted_intent_plan() {
        return format!(
            "{}:{}:{}",
            payload.semantic_code(),
            family.as_str(),
            entrypoint.as_str()
        );
    }
    payload.semantic_code().to_string()
}

pub(super) fn workflow_runtime_semantics_match_posture(
    posture: WorthQueryWorkflowContributionPosture,
    runtime_semantics: &WorthQueryWorkflowRuntimeSemantics,
) -> bool {
    match posture {
        WorthQueryWorkflowContributionPosture::PreviewOnly => matches!(
            runtime_semantics.binding(),
            WorthQueryWorkflowRuntimeBindingSemantics::PreviewFoundation {
                evaluation_class: WorkflowPreviewEvaluationClass::ReadOnly,
                ..
            }
        ),
        WorthQueryWorkflowContributionPosture::PromotionEligible => matches!(
            runtime_semantics.binding(),
            WorthQueryWorkflowRuntimeBindingSemantics::PreviewFoundation {
                evaluation_class: WorkflowPreviewEvaluationClass::PromotionEligible,
                ..
            }
        ),
        WorthQueryWorkflowContributionPosture::ConfirmationRequired => matches!(
            runtime_semantics.binding(),
            WorthQueryWorkflowRuntimeBindingSemantics::RuntimePreflight { .. }
                | WorthQueryWorkflowRuntimeBindingSemantics::RuntimePreflightBundle { .. }
        ),
        WorthQueryWorkflowContributionPosture::DiscardRequired => matches!(
            runtime_semantics.binding(),
            WorthQueryWorkflowRuntimeBindingSemantics::PreviewFoundation {
                evaluation_class: WorkflowPreviewEvaluationClass::PromotionEligible,
                ..
            }
        ),
    }
}

pub(super) fn missing_workflow_runtime_semantics_denial(
    operation_label: &'static str,
    payload: &WorthQueryWorkflowContributionPayload,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "workflow-preview",
        target_kind,
        request_identity,
        format!(
            "{operation_label} requires runtime workflow semantics for `{}`",
            payload.semantic_code()
        ),
    )
}

#[cfg(test)]
pub(super) fn missing_workflow_lowering_semantics_denial(
    operation_label: &'static str,
    payload: &WorthQueryWorkflowContributionPayload,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "workflow-preview",
        target_kind,
        request_identity,
        format!(
            "{operation_label} requires workflow lowering semantics for `{}`",
            payload.semantic_code()
        ),
    )
}

pub(super) fn inconsistent_workflow_runtime_semantics_denial(
    operation_label: &'static str,
    payload: &WorthQueryWorkflowContributionPayload,
    runtime_semantics: &WorthQueryWorkflowRuntimeSemantics,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics,
        "workflow-preview",
        target_kind,
        request_identity,
        format!(
            "{operation_label} got workflow runtime semantics `{}:{}` that do not match `{}` posture",
            runtime_semantics.declaration_family().as_str(),
            runtime_semantics.authority_target_family().as_str(),
            payload.posture().as_str()
        ),
    )
}

#[cfg(test)]
pub(super) fn inconsistent_workflow_lowering_semantics_denial(
    operation_label: &'static str,
    payload: &WorthQueryWorkflowContributionPayload,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics,
        "workflow-preview",
        target_kind,
        request_identity,
        format!(
            "{operation_label} got workflow lowering semantics that do not match `{}` workflow runtime semantics",
            payload.posture().as_str()
        ),
    )
}

#[cfg(test)]
pub(super) fn workflow_lowering_semantics_match_runtime(
    runtime_semantics: &WorthQueryWorkflowRuntimeSemantics,
    lowering_semantics: &WorthQueryWorkflowLoweringSemantics,
) -> bool {
    match lowering_semantics {
        WorthQueryWorkflowLoweringSemantics::Mutation { .. } => {
            runtime_semantics.declaration_family()
                == &crate::workflow::WorkflowDeclarationFamily::MutationLoweringNarrow
                && runtime_semantics.authority_target_family()
                    == &crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation
        }
        WorthQueryWorkflowLoweringSemantics::Merge { .. } => {
            runtime_semantics.declaration_family()
                == &crate::workflow::WorkflowDeclarationFamily::MergeLoweringNarrow
                && runtime_semantics.authority_target_family()
                    == &crate::workflow::WorkflowAuthorityTargetFamily::RelationalMerge
        }
        WorthQueryWorkflowLoweringSemantics::Writeback { .. } => {
            runtime_semantics.declaration_family()
                == &crate::workflow::WorkflowDeclarationFamily::WritebackLoweringNarrow
                && runtime_semantics.authority_target_family()
                    == &crate::workflow::WorkflowAuthorityTargetFamily::BridgeWriteback
        }
    }
}

#[cfg(test)]
pub(super) fn workflow_inspection_semantics_match_runtime(
    runtime_semantics: &WorthQueryWorkflowRuntimeSemantics,
    inspection_semantics: &WorthQueryWorkflowInspectionSemantics,
) -> bool {
    match inspection_semantics {
        WorthQueryWorkflowInspectionSemantics::MergeConflict { .. } => {
            runtime_semantics.declaration_family()
                == &crate::workflow::WorkflowDeclarationFamily::ConflictInspectionNarrow
                && runtime_semantics.authority_target_family()
                    == &crate::workflow::WorkflowAuthorityTargetFamily::QueryInspection
        }
        WorthQueryWorkflowInspectionSemantics::PostMergeFromMerge { .. }
        | WorthQueryWorkflowInspectionSemantics::PostMergeFromWriteback { .. } => {
            runtime_semantics.declaration_family()
                == &crate::workflow::WorkflowDeclarationFamily::PostMergeInspectionNarrow
                && runtime_semantics.authority_target_family()
                    == &crate::workflow::WorkflowAuthorityTargetFamily::QueryInspection
        }
    }
}
