use crate::application::{
    WorthQueryDeclarationEntryContributionEvidence,
    WorthQueryDeclarationEntryContributionEvidenceRecord,
    WorthQueryDeclarationEntryContributionTargetFamily,
};
use crate::contribution_composed_orchestration::artifact::WorthQueryContributionComposedContribution;
use crate::contribution_composed_orchestration::intent_result::{
    WorthQueryContributionComposedIntentClassification,
    WorthQueryContributionComposedIntentRequestDescriptor,
    WorthQueryContributionComposedIntentResult, WorthQueryContributionComposedIntentStageResult,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilityTargetBinding,
    WorthQueryInstalledDeclarationContributionTarget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(super) fn denied_result(
    request: WorthQueryContributionComposedIntentRequestDescriptor,
    evaluation: WorthQueryContributionComposedIntentStageResult,
) -> WorthQueryContributionComposedIntentResult {
    WorthQueryContributionComposedIntentResult::new(
        request,
        evaluation,
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentClassification::Denied,
        None,
    )
}

pub(super) fn stale_result(
    request: WorthQueryContributionComposedIntentRequestDescriptor,
    detail: String,
) -> WorthQueryContributionComposedIntentResult {
    WorthQueryContributionComposedIntentResult::new(
        request,
        WorthQueryContributionComposedIntentStageResult::stale(detail),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentClassification::Stale,
        None,
    )
}

pub(super) fn rebind_required_result(
    request: WorthQueryContributionComposedIntentRequestDescriptor,
    detail: String,
) -> WorthQueryContributionComposedIntentResult {
    WorthQueryContributionComposedIntentResult::new(
        request,
        WorthQueryContributionComposedIntentStageResult::rebind_required(detail),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentClassification::RebindRequired,
        None,
    )
}

pub(super) fn failed_result(
    request: WorthQueryContributionComposedIntentRequestDescriptor,
    detail: String,
) -> WorthQueryContributionComposedIntentResult {
    WorthQueryContributionComposedIntentResult::new(
        request,
        WorthQueryContributionComposedIntentStageResult::failed(detail),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentClassification::Failed,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn retained_after_admission_result(
    request: WorthQueryContributionComposedIntentRequestDescriptor,
    evaluation: WorthQueryContributionComposedIntentStageResult,
    admission: WorthQueryContributionComposedIntentStageResult,
    materialization: WorthQueryContributionComposedIntentStageResult,
    contribution: WorthQueryContributionComposedContribution,
) -> WorthQueryContributionComposedIntentResult {
    WorthQueryContributionComposedIntentResult::new(
        request,
        evaluation,
        admission,
        materialization,
        WorthQueryContributionComposedIntentClassification::MaterializationFailedAfterAdmission,
        Some(contribution),
    )
}

pub(super) fn evidence_from_admitted<P>(
    admitted: &crate::domain_capabilities::WorthQueryAdmittedDomainCapabilityContribution<
        P,
        WorthQueryInstalledDeclarationContributionTarget,
    >,
) -> Result<WorthQueryDeclarationEntryContributionEvidence, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
{
    let payload = admitted.payload().payload();
    let record = WorthQueryDeclarationEntryContributionEvidenceRecord {
        target_family: WorthQueryDeclarationEntryContributionTargetFamily::DeclarationBound,
        target_digest: admitted.payload().target().target_digest().to_string(),
        target_binding_digest: admitted.payload().target().binding_digest().to_string(),
        evidence_digest: admitted.admitted_for_reporting()?,
        posture_label: payload.posture_label().to_string(),
        semantic_code: payload.semantic_code().to_string(),
        detail: payload.detail().to_string(),
        decision_stage: None,
    };
    Ok(match payload.category() {
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::Admission => {
            WorthQueryDeclarationEntryContributionEvidence::Admission(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::SupportTraceability => {
            WorthQueryDeclarationEntryContributionEvidence::SupportTraceability(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::ExplanationInspection => {
            WorthQueryDeclarationEntryContributionEvidence::ExplanationInspection(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::WorkflowPreview => {
            WorthQueryDeclarationEntryContributionEvidence::WorkflowPreview(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::ContinuityLineage => {
            WorthQueryDeclarationEntryContributionEvidence::ContinuityLineage(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::ConsequenceAftermath => {
            WorthQueryDeclarationEntryContributionEvidence::ConsequenceAftermath(record)
        }
        crate::domain_capabilities::WorthQueryDomainCapabilityCategory::InvariantCapability => {
            WorthQueryDeclarationEntryContributionEvidence::SupportTraceability(record)
        }
    })
}
