use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};
use super::graph_semantics::{
    compose_graph_capability_identity, compose_graph_invariant_denial_identity,
    WorthQueryGraphCapabilityRuntimeSemantics, WorthQueryGraphInvariantDenialRuntimeSemantics,
};
use super::posture::WorthQueryInvariantCapabilityContributionPosture;
use super::registration_semantics::{
    compose_invariant_registration_identity, WorthQueryInvariantRegistrationRuntimeSemantics,
};

fn compose_invariant_capability_payload_identity(
    posture: WorthQueryInvariantCapabilityContributionPosture,
    semantic_code: &str,
    detail: &str,
    graph_capability: Option<&WorthQueryGraphCapabilityRuntimeSemantics>,
    graph_invariant_denial: Option<&WorthQueryGraphInvariantDenialRuntimeSemantics>,
    invariant_registration: Option<&WorthQueryInvariantRegistrationRuntimeSemantics>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_payload_v3")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            WorthQueryDomainCapabilityCategory::InvariantCapability.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail);
    identity = match graph_capability {
        Some(semantics) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("graph_capability"),
            &compose_graph_capability_identity(semantics)?,
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("graph_capability"), "none"),
    };
    identity = match graph_invariant_denial {
        Some(semantics) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("graph_invariant_denial"),
            &compose_graph_invariant_denial_identity(semantics)?,
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("graph_invariant_denial"), "none"),
    };
    identity = match invariant_registration {
        Some(semantics) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("invariant_registration"),
            &compose_invariant_registration_identity(semantics),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("invariant_registration"), "none"),
    };
    Ok(identity.seal()?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantCapabilityContributionPayload {
    posture: WorthQueryInvariantCapabilityContributionPosture,
    semantic_code: String,
    detail: String,
    graph_capability: Option<WorthQueryGraphCapabilityRuntimeSemantics>,
    graph_invariant_denial: Option<WorthQueryGraphInvariantDenialRuntimeSemantics>,
    invariant_registration: Option<WorthQueryInvariantRegistrationRuntimeSemantics>,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInvariantCapabilityContributionPayload {
    pub fn new(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_graph_capability(
            posture,
            semantic_code,
            detail,
            None,
        )?)
    }

    pub fn with_graph_capability(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        graph_capability: Option<WorthQueryGraphCapabilityRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            posture,
            semantic_code,
            detail,
            graph_capability,
            None,
            None,
        )?)
    }

    pub fn with_graph_invariant_denial(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        graph_invariant_denial: Option<WorthQueryGraphInvariantDenialRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            posture,
            semantic_code,
            detail,
            None,
            graph_invariant_denial,
            None,
        )?)
    }

    pub fn with_invariant_registration(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        invariant_registration: Option<WorthQueryInvariantRegistrationRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            posture,
            semantic_code,
            detail,
            None,
            None,
            invariant_registration,
        )?)
    }

    pub fn with_runtime_semantics(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        graph_capability: Option<WorthQueryGraphCapabilityRuntimeSemantics>,
        graph_invariant_denial: Option<WorthQueryGraphInvariantDenialRuntimeSemantics>,
        invariant_registration: Option<WorthQueryInvariantRegistrationRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity = compose_invariant_capability_payload_identity(
            posture,
            &semantic_code,
            &detail,
            graph_capability.as_ref(),
            graph_invariant_denial.as_ref(),
            invariant_registration.as_ref(),
        )?;
        Ok(Self {
            posture,
            semantic_code,
            detail,
            graph_capability,
            graph_invariant_denial,
            invariant_registration,
            payload_identity,
        })
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        WorthQueryDomainCapabilityCategory::InvariantCapability
    }

    pub fn posture(&self) -> WorthQueryInvariantCapabilityContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn graph_capability(&self) -> Option<&WorthQueryGraphCapabilityRuntimeSemantics> {
        self.graph_capability.as_ref()
    }

    pub fn graph_invariant_denial(
        &self,
    ) -> Option<&WorthQueryGraphInvariantDenialRuntimeSemantics> {
        self.graph_invariant_denial.as_ref()
    }

    pub fn invariant_registration(
        &self,
    ) -> Option<&WorthQueryInvariantRegistrationRuntimeSemantics> {
        self.invariant_registration.as_ref()
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

impl SealedPayload for WorthQueryInvariantCapabilityContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryInvariantCapabilityContributionPayload {
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
