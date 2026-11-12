use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope;

use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::runtime::{
    CausalEvidenceFamily, CausalEvidenceReferenceSet, CausalInspectionExplanationFamily,
    CausalInspectionMaterializationPolicy, CausalInspectionRedactionPolicy,
    CausalInspectionRichness, CausalInspectionTarget,
};

use super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExplanationContributionPosture {
    RequiresContext,
    ExplainsFallback,
    ExplainsAmbiguity,
}

impl WorthQueryExplanationContributionPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequiresContext => "requires-context",
            Self::ExplainsFallback => "explains-fallback",
            Self::ExplainsAmbiguity => "explains-ambiguity",
        }
    }

    pub const fn semantic_posture(self) -> WorthQueryDomainCapabilitySemanticPosture {
        match self {
            Self::RequiresContext => {
                WorthQueryDomainCapabilitySemanticPosture::ExplanationRequiresContext
            }
            Self::ExplainsFallback => {
                WorthQueryDomainCapabilitySemanticPosture::ExplanationExplainsFallback
            }
            Self::ExplainsAmbiguity => {
                WorthQueryDomainCapabilitySemanticPosture::ExplanationExplainsAmbiguity
            }
        }
    }
}

fn compose_explanation_runtime_semantics_identity(
    reference_set: &CausalEvidenceReferenceSet,
    target: &CausalInspectionTarget,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: &[CausalEvidenceFamily],
    bridge_envelope: Option<&BridgeCausalExplanationEnvelope>,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder(
        "worth_query_domain_capability_explanation_runtime_semantics_v1",
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("reference_set"),
        reference_set.reference_set_digest().evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("target"),
        target.target_identity().evidence_identity(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("family"),
        explanation_family.as_str(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("richness"),
        requested_richness.as_str(),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("evidence_families"),
        requested_evidence_families
            .iter()
            .map(CausalEvidenceFamily::as_str),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("redaction"),
        redaction_policy.as_str(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("materialization"),
        materialization_policy.as_str(),
    );
    identity = match bridge_envelope {
        Some(envelope) => identity.field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_envelope"),
            envelope.envelope_evidence_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("bridge_envelope"), "none"),
    };
    identity.seal()
}

fn compose_explanation_payload_identity(
    posture: WorthQueryExplanationContributionPosture,
    semantic_code: &str,
    detail: &str,
    runtime_semantics: Option<&WorthQueryExplanationRuntimeSemantics>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_payload_v2")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            WorthQueryDomainCapabilityCategory::ExplanationInspection.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail);
    identity = match runtime_semantics {
        Some(semantics) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime_semantics"),
            semantics.semantics_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("runtime_semantics"), "none"),
    };
    identity.seal()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExplanationRuntimeSemantics {
    reference_set: CausalEvidenceReferenceSet,
    target: CausalInspectionTarget,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: Vec<CausalEvidenceFamily>,
    bridge_envelope: Option<BridgeCausalExplanationEnvelope>,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
    semantics_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryExplanationRuntimeSemantics {
    pub fn new(
        reference_set: CausalEvidenceReferenceSet,
        target: CausalInspectionTarget,
        explanation_family: CausalInspectionExplanationFamily,
        requested_richness: CausalInspectionRichness,
        requested_evidence_families: Vec<CausalEvidenceFamily>,
        bridge_envelope: Option<BridgeCausalExplanationEnvelope>,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantics_identity = compose_explanation_runtime_semantics_identity(
            &reference_set,
            &target,
            explanation_family,
            requested_richness,
            &requested_evidence_families,
            bridge_envelope.as_ref(),
            redaction_policy,
            materialization_policy,
        )?;
        Ok(Self {
            reference_set,
            target,
            explanation_family,
            requested_richness,
            requested_evidence_families,
            bridge_envelope,
            redaction_policy,
            materialization_policy,
            semantics_identity,
        })
    }

    pub fn reference_set(&self) -> &CausalEvidenceReferenceSet {
        &self.reference_set
    }

    pub fn target(&self) -> &CausalInspectionTarget {
        &self.target
    }

    pub fn explanation_family(&self) -> CausalInspectionExplanationFamily {
        self.explanation_family
    }

    pub fn requested_richness(&self) -> CausalInspectionRichness {
        self.requested_richness
    }

    pub fn requested_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.requested_evidence_families
    }

    pub fn bridge_envelope(&self) -> Option<&BridgeCausalExplanationEnvelope> {
        self.bridge_envelope.as_ref()
    }

    pub fn redaction_policy(&self) -> CausalInspectionRedactionPolicy {
        self.redaction_policy
    }

    pub fn materialization_policy(&self) -> CausalInspectionMaterializationPolicy {
        self.materialization_policy
    }

    pub(in crate::domain_capabilities) fn semantics_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.semantics_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExplanationContributionPayload {
    posture: WorthQueryExplanationContributionPosture,
    semantic_code: String,
    detail: String,
    runtime_semantics: Option<WorthQueryExplanationRuntimeSemantics>,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryExplanationContributionPayload {
    pub fn new(
        posture: WorthQueryExplanationContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(posture, semantic_code, detail, None)
    }

    pub fn with_runtime_semantics(
        posture: WorthQueryExplanationContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryExplanationRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity = compose_explanation_payload_identity(
            posture,
            &semantic_code,
            &detail,
            runtime_semantics.as_ref(),
        )?;
        Ok(Self {
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            payload_identity,
        })
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        WorthQueryDomainCapabilityCategory::ExplanationInspection
    }

    pub fn posture(&self) -> WorthQueryExplanationContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn runtime_semantics(&self) -> Option<&WorthQueryExplanationRuntimeSemantics> {
        self.runtime_semantics.as_ref()
    }

    pub fn payload_digest(&self) -> &str {
        self.payload_identity.as_str()
    }
}

impl SealedPayload for WorthQueryExplanationContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryExplanationContributionPayload {
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
