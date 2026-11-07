use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAdmissionContributionPosture {
    Advisory,
    Violation,
    SupportOnly,
}

impl WorthQueryAdmissionContributionPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::Violation => "violation",
            Self::SupportOnly => "support-only",
        }
    }

    pub const fn semantic_posture(self) -> WorthQueryDomainCapabilitySemanticPosture {
        match self {
            Self::Advisory => WorthQueryDomainCapabilitySemanticPosture::AdmissionAdvisory,
            Self::Violation => WorthQueryDomainCapabilitySemanticPosture::AdmissionViolation,
            Self::SupportOnly => WorthQueryDomainCapabilitySemanticPosture::AdmissionSupportOnly,
        }
    }

    pub const fn default_decision_stage(self) -> &'static str {
        match self {
            Self::Advisory => "domain_capability_advisory",
            Self::Violation => "domain_capability_violation",
            Self::SupportOnly => "domain_capability_support_only",
        }
    }
}

fn compose_admission_payload_identity(
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: &str,
    detail: &str,
    decision_stage: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_payload_v3")
            .field_shape(
                WorthQueryEvidenceTag::new("category"),
                WorthQueryDomainCapabilityCategory::Admission.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
            .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
            .field_shape(WorthQueryEvidenceTag::new("detail"), detail)
            .field_shape(WorthQueryEvidenceTag::new("decision_stage"), decision_stage)
            .seal()?,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmissionContributionPayload {
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: String,
    detail: String,
    decision_stage: &'static str,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryAdmissionContributionPayload {
    pub fn new(
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_decision_stage(
            posture,
            semantic_code,
            detail,
            posture.default_decision_stage(),
        )?)
    }

    pub fn with_decision_stage(
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        decision_stage: &'static str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity =
            compose_admission_payload_identity(posture, &semantic_code, &detail, decision_stage)?;
        Ok(Self {
            posture,
            semantic_code,
            detail,
            decision_stage,
            payload_identity,
        })
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        WorthQueryDomainCapabilityCategory::Admission
    }

    pub fn posture(&self) -> WorthQueryAdmissionContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn decision_stage(&self) -> &'static str {
        self.decision_stage
    }

    pub fn payload_digest(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_for_reporting(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}

impl SealedPayload for WorthQueryAdmissionContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryAdmissionContributionPayload {
    fn category(&self) -> WorthQueryDomainCapabilityCategory {
        self.category()
    }

    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }

    fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture {
        self.posture().semantic_posture()
    }

    fn semantic_code(&self) -> &str {
        self.semantic_code()
    }

    fn detail(&self) -> &str {
        self.detail()
    }

    fn payload_digest(&self) -> &str {
        self.payload_digest()
    }

    fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}
