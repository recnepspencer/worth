use crate::application::{
    WorthQueryDeclarationAspectPublication, WorthQueryDeclarationBridgeAuthorityAspectSummary,
    WorthQueryDeclarationBridgeContinuationFamily, WorthQueryDeclarationBridgeContinuationMode,
    WorthQueryDeclarationBridgeRoutingClass, WorthQueryDeclarationBridgeRoutingDenialCause,
    WorthQueryDeclarationBridgeTruthContext, WorthQueryDeclarationEnvelopeClass,
    WorthQueryDeclarationEnvelopeEvidenceOrigin, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationReceiptDenialCause,
    WorthQueryDeclarationRelationalAuthorityAspectSummary,
    WorthQueryDeclarationRelationalAuthorityFamily, WorthQueryDeclarationRelationalRoutingClass,
    WorthQueryDeclarationRelationalRoutingDenialCause, WorthQueryDeclarationRelationalTruthClaim,
    WorthQueryDeclarationRoutePlanDenialCause, WorthQueryDeclarationSignalAuthorityAspectSummary,
    WorthQueryDeclarationSignalCompatibilityClass,
    WorthQueryDeclarationSignalCompatibilityDenialCause,
    WorthQueryDeclarationSignalExecutionFamily, WorthQueryDomainEntryMarker,
};
use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::{
    contribution::{
        WorthQueryDeclarationEntryContributionComposition,
        WorthQueryDeclarationEntryContributionCompositionError,
    },
    support::WorthQueryDeclarationEntryReadinessReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationEntryInspectionRelationalPosture {
    pub(crate) class: WorthQueryDeclarationRelationalRoutingClass,
    pub(crate) truth_claim: Option<WorthQueryDeclarationRelationalTruthClaim>,
    pub(crate) authority_family: Option<WorthQueryDeclarationRelationalAuthorityFamily>,
    pub(crate) authority_aspect_summary: WorthQueryDeclarationRelationalAuthorityAspectSummary,
    pub(crate) routing_digest: String,
    pub(crate) denial_cause: Option<WorthQueryDeclarationRelationalRoutingDenialCause>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationEntryInspectionBridgePosture {
    pub(crate) class: WorthQueryDeclarationBridgeRoutingClass,
    pub(crate) continuation_mode: Option<WorthQueryDeclarationBridgeContinuationMode>,
    pub(crate) truth_context: Option<WorthQueryDeclarationBridgeTruthContext>,
    pub(crate) continuation_family: Option<WorthQueryDeclarationBridgeContinuationFamily>,
    pub(crate) authority_aspect_summary: WorthQueryDeclarationBridgeAuthorityAspectSummary,
    pub(crate) routing_digest: String,
    pub(crate) denial_cause: Option<WorthQueryDeclarationBridgeRoutingDenialCause>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationEntryInspectionSignalPosture {
    pub(crate) class: WorthQueryDeclarationSignalCompatibilityClass,
    pub(crate) execution_family: Option<WorthQueryDeclarationSignalExecutionFamily>,
    pub(crate) basis_families: Vec<BasisFamily>,
    pub(crate) authority_aspect_summary: WorthQueryDeclarationSignalAuthorityAspectSummary,
    pub(crate) compatibility_digest: String,
    pub(crate) denial_cause: Option<WorthQueryDeclarationSignalCompatibilityDenialCause>,
}

pub struct WorthQueryDeclarationEntryInspection<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    pub(crate) declaration_family_key: &'static str,
    pub(crate) handle_identity_digest: String,
    pub(crate) operating_context_identity_digest: String,
    pub(crate) declaration_digest: String,
    pub(crate) progression_digest: Option<String>,
    pub(crate) route_plan_digest: Option<String>,
    pub(crate) receipt_digest: Option<String>,
    pub(crate) envelope_digest: String,
    pub(crate) envelope_class: WorthQueryDeclarationEnvelopeClass,
    pub(crate) envelope_aspect_publication: WorthQueryDeclarationAspectPublication,
    pub(crate) evidence_origin: WorthQueryDeclarationEnvelopeEvidenceOrigin,
    pub(crate) route_denial_cause: Option<WorthQueryDeclarationRoutePlanDenialCause>,
    pub(crate) receipt_denial_cause: Option<WorthQueryDeclarationReceiptDenialCause>,
    pub(crate) route_reason: Option<String>,
    pub(crate) receipt_reason: String,
    pub(crate) relational_posture: Option<WorthQueryDeclarationEntryInspectionRelationalPosture>,
    pub(crate) bridge_posture: Option<WorthQueryDeclarationEntryInspectionBridgePosture>,
    pub(crate) signal_posture: Option<WorthQueryDeclarationEntryInspectionSignalPosture>,
    pub(crate) contribution_composition: Option<WorthQueryDeclarationEntryContributionComposition>,
    pub(crate) matching_row_digests: Vec<String>,
    pub(crate) readiness: WorthQueryDeclarationEntryReadinessReport<D, I>,
    pub(crate) inspection_digest: String,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationEntryInspection<D, I>
{
    pub fn declaration_family_key(&self) -> &'static str {
        self.declaration_family_key
    }
    pub fn handle_identity_digest(&self) -> &str {
        &self.handle_identity_digest
    }
    pub fn operating_context_identity_digest(&self) -> &str {
        &self.operating_context_identity_digest
    }
    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }
    pub fn progression_digest(&self) -> Option<&str> {
        self.progression_digest.as_deref()
    }
    pub fn route_plan_digest(&self) -> Option<&str> {
        self.route_plan_digest.as_deref()
    }
    pub fn receipt_digest(&self) -> Option<&str> {
        self.receipt_digest.as_deref()
    }
    pub fn envelope_digest(&self) -> &str {
        &self.envelope_digest
    }
    pub fn envelope_class(&self) -> WorthQueryDeclarationEnvelopeClass {
        self.envelope_class
    }
    pub fn envelope_aspect_publication(&self) -> &WorthQueryDeclarationAspectPublication {
        &self.envelope_aspect_publication
    }
    pub fn evidence_origin(&self) -> WorthQueryDeclarationEnvelopeEvidenceOrigin {
        self.evidence_origin
    }
    pub fn route_denial_cause(&self) -> Option<WorthQueryDeclarationRoutePlanDenialCause> {
        self.route_denial_cause
    }
    pub fn receipt_denial_cause(&self) -> Option<WorthQueryDeclarationReceiptDenialCause> {
        self.receipt_denial_cause
    }
    pub fn route_reason(&self) -> Option<&str> {
        self.route_reason.as_deref()
    }
    pub fn receipt_reason(&self) -> &str {
        &self.receipt_reason
    }
    pub fn relational_posture(
        &self,
    ) -> Option<&WorthQueryDeclarationEntryInspectionRelationalPosture> {
        self.relational_posture.as_ref()
    }
    pub fn bridge_posture(&self) -> Option<&WorthQueryDeclarationEntryInspectionBridgePosture> {
        self.bridge_posture.as_ref()
    }
    pub fn signal_posture(&self) -> Option<&WorthQueryDeclarationEntryInspectionSignalPosture> {
        self.signal_posture.as_ref()
    }
    pub fn contribution_composition(
        &self,
    ) -> Option<&WorthQueryDeclarationEntryContributionComposition> {
        self.contribution_composition.as_ref()
    }
    pub fn matching_row_digests(&self) -> &[String] {
        &self.matching_row_digests
    }
    pub fn readiness(&self) -> &WorthQueryDeclarationEntryReadinessReport<D, I> {
        &self.readiness
    }
    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }
}

impl WorthQueryDeclarationEntryInspectionRelationalPosture {
    pub fn class(&self) -> WorthQueryDeclarationRelationalRoutingClass {
        self.class
    }

    pub fn truth_claim(&self) -> Option<WorthQueryDeclarationRelationalTruthClaim> {
        self.truth_claim
    }

    pub fn authority_family(&self) -> Option<WorthQueryDeclarationRelationalAuthorityFamily> {
        self.authority_family
    }

    pub fn routing_digest(&self) -> &str {
        &self.routing_digest
    }

    pub fn denial_cause(&self) -> Option<WorthQueryDeclarationRelationalRoutingDenialCause> {
        self.denial_cause
    }

    pub fn aspect_summary(&self) -> &WorthQueryDeclarationRelationalAuthorityAspectSummary {
        &self.authority_aspect_summary
    }
}

impl WorthQueryDeclarationEntryInspectionBridgePosture {
    pub fn class(&self) -> WorthQueryDeclarationBridgeRoutingClass {
        self.class
    }

    pub fn continuation_mode(&self) -> Option<WorthQueryDeclarationBridgeContinuationMode> {
        self.continuation_mode
    }

    pub fn truth_context(&self) -> Option<WorthQueryDeclarationBridgeTruthContext> {
        self.truth_context
    }

    pub fn continuation_family(&self) -> Option<WorthQueryDeclarationBridgeContinuationFamily> {
        self.continuation_family
    }

    pub fn routing_digest(&self) -> &str {
        &self.routing_digest
    }

    pub fn denial_cause(&self) -> Option<WorthQueryDeclarationBridgeRoutingDenialCause> {
        self.denial_cause
    }

    pub fn aspect_summary(&self) -> &WorthQueryDeclarationBridgeAuthorityAspectSummary {
        &self.authority_aspect_summary
    }
}

impl WorthQueryDeclarationEntryInspectionSignalPosture {
    pub fn class(&self) -> WorthQueryDeclarationSignalCompatibilityClass {
        self.class
    }

    pub fn execution_family(&self) -> Option<WorthQueryDeclarationSignalExecutionFamily> {
        self.execution_family
    }

    pub fn basis_families(&self) -> &[BasisFamily] {
        &self.basis_families
    }

    pub fn compatibility_digest(&self) -> &str {
        &self.compatibility_digest
    }

    pub fn denial_cause(&self) -> Option<WorthQueryDeclarationSignalCompatibilityDenialCause> {
        self.denial_cause
    }

    pub fn aspect_summary(&self) -> &WorthQueryDeclarationSignalAuthorityAspectSummary {
        &self.authority_aspect_summary
    }
}

pub enum WorthQueryDeclarationEntryInspectionError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    RetainedSubjectMismatch {
        declaration_family_key: &'static str,
        reason: &'static str,
        _marker: std::marker::PhantomData<(D, I)>,
    },
    ContributionComposition(WorthQueryDeclarationEntryContributionCompositionError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationEntryInspectionError<D, I>
{
    #[cfg(test)]
    pub(crate) fn new(declaration_family_key: &'static str, reason: &'static str) -> Self {
        Self::RetainedSubjectMismatch {
            declaration_family_key,
            reason,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn declaration_family_key(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => I::Family::semantic_family_key(),
            Self::RetainedSubjectMismatch {
                declaration_family_key,
                ..
            } => declaration_family_key,
            Self::ContributionComposition(error) => error.declaration_family_key(),
        }
    }
    pub fn reason(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "canonical evidence identity refused",
            Self::RetainedSubjectMismatch { reason, .. } => reason,
            Self::ContributionComposition(error) => error.reason(),
        }
    }
    pub fn contribution_composition_error(
        &self,
    ) -> Option<&WorthQueryDeclarationEntryContributionCompositionError<D, I>> {
        match self {
            Self::ContributionComposition(error) => Some(error),
            Self::EvidenceIdentityDerivationDenied(_) | Self::RetainedSubjectMismatch { .. } => {
                None
            }
        }
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryInspectionError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
