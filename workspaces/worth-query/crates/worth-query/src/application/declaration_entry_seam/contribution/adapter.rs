use crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::{
    WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDeclarationEntryContributionEvidence,
    WorthQueryDeclarationEntryContributionEvidenceRecord,
    WorthQueryDeclarationEntryContributionTargetFamily,
};

macro_rules! impl_try_from_admitted {
    ($wrapper:path, $target:ty, $variant:ident, $family:expr) => {
        impl TryFrom<$wrapper> for WorthQueryDeclarationEntryContributionEvidence {
            type Error = IdentityDerivationDenial;

            fn try_from(value: $wrapper) -> Result<Self, Self::Error> {
                let payload = value.payload().payload();
                Ok(Self::$variant(
                    WorthQueryDeclarationEntryContributionEvidenceRecord {
                        target_family: target_family_for::<$target>(),
                        target_digest: value.payload().target().target_digest().to_string(),
                        target_binding_digest: value
                            .payload()
                            .target()
                            .binding_digest()
                            .to_string(),
                        evidence_digest: value.admitted_for_reporting()?,
                        posture_label: payload.posture_label().to_string(),
                        semantic_code: payload.semantic_code().to_string(),
                        detail: payload.detail().to_string(),
                        decision_stage: decision_stage_for(&$family, payload),
                    },
                ))
            }
        }
    };
}

trait EntryContributionPayloadView {
    fn posture_label(&self) -> &'static str;
}

impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQueryAdmissionContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}
impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQuerySupportContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}
impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQueryExplanationContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}
impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQueryWorkflowContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}
impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQueryContinuityContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}
impl EntryContributionPayloadView
    for crate::domain_capabilities::WorthQueryAftermathContributionPayload
{
    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }
}

fn target_family_for<T: WorthQueryDomainCapabilityTargetBinding>(
) -> WorthQueryDeclarationEntryContributionTargetFamily {
    let kind = std::any::type_name::<T>();
    if kind.ends_with("WorthQueryDeclarationBoundContributionTarget") {
        WorthQueryDeclarationEntryContributionTargetFamily::DeclarationBound
    } else if kind.ends_with("WorthQueryAdmittedPlanBoundContributionTarget") {
        WorthQueryDeclarationEntryContributionTargetFamily::AdmittedPlanBound
    } else {
        WorthQueryDeclarationEntryContributionTargetFamily::LowerRuntimeBound
    }
}

fn decision_stage_for<P: EntryContributionPayloadView>(
    _category: &WorthQueryDeclarationEntryContributionCategoryFamily,
    _payload: &P,
) -> Option<String> {
    None
}

impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedAdmissionContribution<
        crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    Admission,
    WorthQueryDeclarationEntryContributionCategoryFamily::Admission
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedAdmissionContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    Admission,
    WorthQueryDeclarationEntryContributionCategoryFamily::Admission
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedSupportContribution<
        crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    SupportTraceability,
    WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedSupportContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    SupportTraceability,
    WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedSupportContribution<
        crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    SupportTraceability,
    WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedExplanationContribution<
        crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    ExplanationInspection,
    WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedExplanationContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    ExplanationInspection,
    WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedExplanationContribution<
        crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ExplanationInspection,
    WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedWorkflowContribution<
        crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget,
    WorkflowPreview,
    WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedWorkflowContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    WorkflowPreview,
    WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedContinuityContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    ContinuityLineage,
    WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedAftermathContribution<
        crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryAdmittedPlanBoundContributionTarget,
    ConsequenceAftermath,
    WorthQueryDeclarationEntryContributionCategoryFamily::ConsequenceAftermath
);
impl_try_from_admitted!(
    crate::domain_capabilities::WorthQueryAdmittedAftermathContribution<
        crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ConsequenceAftermath,
    WorthQueryDeclarationEntryContributionCategoryFamily::ConsequenceAftermath
);
