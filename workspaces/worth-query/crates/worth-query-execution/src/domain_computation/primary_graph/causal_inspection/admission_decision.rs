mod subject;
pub use subject::CausalInspectionAdmissionSubject;

use super::identity::{
    compose_causal_admission_decision_identity, CausalInspectionAdmissionDecisionIdentity,
};
use super::inventory::CausalEvidenceFamily;
use super::request::{CausalInspectionRequest, CausalInspectionRichness};
use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceIdentityDerivationDenial, WorthQueryEvidenceScope,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionAdmissionDecisionKind {
    Success,
    Advisory,
    Violation,
}

impl CausalInspectionAdmissionDecisionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Advisory => "advisory",
            Self::Violation => "violation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionAdvisoryKind {
    MaterializedDetailDeferredUntilBridgeEnvelope,
}

impl CausalInspectionAdvisoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MaterializedDetailDeferredUntilBridgeEnvelope => {
                "materialized_detail_deferred_until_bridge_envelope"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionViolationKind {
    UnsupportedExplanationFamily,
}

impl CausalInspectionViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedExplanationFamily => "unsupported_explanation_family",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionAdmissionDecision {
    kind: CausalInspectionAdmissionDecisionKind,
    advisory_kind: Option<CausalInspectionAdvisoryKind>,
    violation_kind: Option<CausalInspectionViolationKind>,
    admitted_richness: CausalInspectionRichness,
    permitted_evidence_families: Vec<CausalEvidenceFamily>,
    decision_identity: CausalInspectionAdmissionDecisionIdentity,
}

impl CausalInspectionAdmissionDecision {
    pub(super) fn success(
        request: &CausalInspectionRequest,
    ) -> Result<Self, WorthQueryEvidenceIdentityDerivationDenial> {
        Self::new(
            CausalInspectionAdmissionDecisionKind::Success,
            None,
            None,
            request.requested_richness(),
            request.requested_evidence_families().to_vec(),
        )
    }

    pub(super) fn advisory(
        request: &CausalInspectionRequest,
        advisory_kind: CausalInspectionAdvisoryKind,
    ) -> Result<Self, WorthQueryEvidenceIdentityDerivationDenial> {
        Self::new(
            CausalInspectionAdmissionDecisionKind::Advisory,
            Some(advisory_kind),
            None,
            CausalInspectionRichness::ReferenceOnly,
            request.requested_evidence_families().to_vec(),
        )
    }

    pub(super) fn violation(
        request: &CausalInspectionRequest,
        violation_kind: CausalInspectionViolationKind,
    ) -> Result<Self, WorthQueryEvidenceIdentityDerivationDenial> {
        Self::new(
            CausalInspectionAdmissionDecisionKind::Violation,
            None,
            Some(violation_kind),
            CausalInspectionRichness::ReferenceOnly,
            request.requested_evidence_families().to_vec(),
        )
    }

    fn new(
        kind: CausalInspectionAdmissionDecisionKind,
        advisory_kind: Option<CausalInspectionAdvisoryKind>,
        violation_kind: Option<CausalInspectionViolationKind>,
        admitted_richness: CausalInspectionRichness,
        permitted_evidence_families: Vec<CausalEvidenceFamily>,
    ) -> Result<Self, WorthQueryEvidenceIdentityDerivationDenial> {
        let mut decision = Self {
            kind,
            advisory_kind,
            violation_kind,
            admitted_richness,
            permitted_evidence_families,
            decision_identity: CausalInspectionAdmissionDecisionIdentity::from(
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::CausalInspectionAdmissionDecision,
                )
                .seal()?,
            ),
        };
        decision.decision_identity = compose_causal_admission_decision_identity(&decision)?;
        Ok(decision)
    }

    pub fn kind(&self) -> CausalInspectionAdmissionDecisionKind {
        self.kind
    }

    pub fn advisory_kind(&self) -> Option<CausalInspectionAdvisoryKind> {
        self.advisory_kind
    }

    pub fn violation_kind(&self) -> Option<CausalInspectionViolationKind> {
        self.violation_kind
    }

    pub fn admitted_richness(&self) -> CausalInspectionRichness {
        self.admitted_richness
    }

    pub fn permitted_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.permitted_evidence_families
    }

    pub fn decision_for_reporting(&self) -> &str {
        self.decision_identity.as_str()
    }

    pub(super) fn decision_identity(&self) -> &CausalInspectionAdmissionDecisionIdentity {
        &self.decision_identity
    }
}
