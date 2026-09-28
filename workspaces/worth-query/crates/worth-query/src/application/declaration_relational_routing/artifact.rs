use crate::application::{
    WorthQueryDeclarationAspectContract, WorthQueryDeclarationAspectCoverage,
    WorthQueryDeclarationAspectCoverageBasis, WorthQueryDeclarationAspectFit,
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationEnvelopeEvidenceOrigin,
    WorthQueryDeclarationInput, WorthQueryDeclarationReceiptDenialCause,
    WorthQueryDeclarationRoutePlanDenialCause, WorthQueryDomainEntryMarker,
};

use super::{
    contract::{
        WorthQueryDeclarationRelationalAuthorityFamily, WorthQueryDeclarationRelationalTruthClaim,
    },
    explain::WorthQueryDeclarationRelationalRoutingExplanation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalRoutingClass {
    ExclusiveRelationalTruth,
    MixedAuthorityRelationalTruth,
}

/// The lower surface a relational truth contract binds to. As with
/// `WorthQueryDeclarationRelationalAuthorityFamily`, "relational" names the
/// truth source, not the crate that owns the surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalBinding {
    Runtime(&'static str),
    History(&'static str),
    GroupedTruth(&'static str),
    CommitStrategies(&'static str),
    BridgeSource(&'static str),
}

impl WorthQueryDeclarationRelationalBinding {
    pub fn surface(&self) -> &'static str {
        match self {
            Self::Runtime(surface)
            | Self::History(surface)
            | Self::GroupedTruth(surface)
            | Self::CommitStrategies(surface)
            | Self::BridgeSource(surface) => surface,
        }
    }
}

pub struct WorthQueryDeclarationRelationalRouting<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    class: WorthQueryDeclarationRelationalRoutingClass,
    truth_claim: WorthQueryDeclarationRelationalTruthClaim,
    authority_family: WorthQueryDeclarationRelationalAuthorityFamily,
    binding: WorthQueryDeclarationRelationalBinding,
    aspect_contract: WorthQueryDeclarationAspectContract,
    aspect_coverage: WorthQueryDeclarationAspectCoverage,
    aspect_coverage_basis: WorthQueryDeclarationAspectCoverageBasis,
    aspect_fit: WorthQueryDeclarationAspectFit,
    envelope: WorthQueryDeclarationEnvelope<D, I>,
    relational_routing_digest: String,
    explanation: WorthQueryDeclarationRelationalRoutingExplanation,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationRelationalRouting<D, I>
{
    pub(crate) fn new(
        class: WorthQueryDeclarationRelationalRoutingClass,
        truth_claim: WorthQueryDeclarationRelationalTruthClaim,
        authority_family: WorthQueryDeclarationRelationalAuthorityFamily,
        binding: WorthQueryDeclarationRelationalBinding,
        aspect_contract: WorthQueryDeclarationAspectContract,
        aspect_coverage: WorthQueryDeclarationAspectCoverage,
        aspect_coverage_basis: WorthQueryDeclarationAspectCoverageBasis,
        aspect_fit: WorthQueryDeclarationAspectFit,
        envelope: WorthQueryDeclarationEnvelope<D, I>,
        relational_routing_digest: String,
        explanation: WorthQueryDeclarationRelationalRoutingExplanation,
    ) -> Self {
        Self {
            class,
            truth_claim,
            authority_family,
            binding,
            aspect_contract,
            aspect_coverage,
            aspect_coverage_basis,
            aspect_fit,
            envelope,
            relational_routing_digest,
            explanation,
        }
    }

    pub fn class(&self) -> WorthQueryDeclarationRelationalRoutingClass {
        self.class
    }

    pub fn truth_claim(&self) -> WorthQueryDeclarationRelationalTruthClaim {
        self.truth_claim
    }

    pub fn authority_family(&self) -> WorthQueryDeclarationRelationalAuthorityFamily {
        self.authority_family
    }

    pub fn binding(&self) -> &WorthQueryDeclarationRelationalBinding {
        &self.binding
    }

    pub fn aspect_contract(&self) -> &WorthQueryDeclarationAspectContract {
        &self.aspect_contract
    }

    pub fn aspect_coverage(&self) -> &WorthQueryDeclarationAspectCoverage {
        &self.aspect_coverage
    }

    pub fn aspect_coverage_basis(&self) -> WorthQueryDeclarationAspectCoverageBasis {
        self.aspect_coverage_basis
    }

    pub fn aspect_fit(&self) -> WorthQueryDeclarationAspectFit {
        self.aspect_fit
    }

    pub fn declaration_family_key(&self) -> &'static str {
        self.envelope.declaration_family_key()
    }

    pub fn handle_identity_digest(&self) -> &str {
        self.envelope.handle_identity_digest()
    }

    pub fn operating_context_identity_digest(&self) -> &str {
        self.envelope.operating_context_identity_digest()
    }

    pub fn declaration_digest(&self) -> &str {
        self.envelope.declaration_digest()
    }

    pub fn progression_digest(&self) -> Option<&str> {
        self.envelope.progression_digest()
    }

    pub fn route_plan_digest(&self) -> Option<&str> {
        self.envelope.route_plan_digest()
    }

    pub fn receipt_digest(&self) -> &worth_foundational::facade::CanonicalDerivedDigest {
        self.envelope.receipt_digest()
    }

    pub fn envelope_digest(&self) -> &worth_foundational::facade::CanonicalDerivedDigest {
        self.envelope.envelope_digest()
    }

    pub fn relational_routing_digest(&self) -> &str {
        &self.relational_routing_digest
    }

    pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
        &self.envelope
    }

    pub fn route_denial_cause(&self) -> Option<WorthQueryDeclarationRoutePlanDenialCause> {
        self.envelope.route_denial_cause()
    }

    pub fn receipt_denial_cause(&self) -> Option<WorthQueryDeclarationReceiptDenialCause> {
        self.envelope.receipt_denial_cause()
    }

    pub fn evidence_origin(&self) -> WorthQueryDeclarationEnvelopeEvidenceOrigin {
        self.envelope.evidence_origin()
    }

    pub fn explain(&self) -> &WorthQueryDeclarationRelationalRoutingExplanation {
        &self.explanation
    }

    #[cfg(test)]
    pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
        self.envelope
    }
}
