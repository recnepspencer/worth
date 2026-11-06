use crate::domain_capabilities::identity::{
    compose_fact_request_entry_digest, domain_capability_scope_encoder,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionConsumptionBindingContext, ProjectionConsumptionSource,
};

use super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathContributionPosture {
    EstablishesFact,
    ConsumesFact,
    DeclaresResidue,
}

impl WorthQueryAftermathContributionPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EstablishesFact => "establishes-fact",
            Self::ConsumesFact => "consumes-fact",
            Self::DeclaresResidue => "declares-residue",
        }
    }

    pub const fn semantic_posture(self) -> WorthQueryDomainCapabilitySemanticPosture {
        match self {
            Self::EstablishesFact => {
                WorthQueryDomainCapabilitySemanticPosture::AftermathEstablishesFact
            }
            Self::ConsumesFact => WorthQueryDomainCapabilitySemanticPosture::AftermathConsumesFact,
            Self::DeclaresResidue => {
                WorthQueryDomainCapabilitySemanticPosture::AftermathDeclaresResidue
            }
        }
    }
}

fn aftermath_source_identity(
    source: &ProjectionConsumptionSource,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_aftermath_source_v1")
        .field_shape(
            WorthQueryEvidenceTag::new("source_family"),
            source.family().as_str(),
        );
    identity = match source.source_identity_handle().evidence_identity() {
        Some(source_identity) => {
            identity.field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        }
        None => identity.field_shape(
            WorthQueryEvidenceTag::new("source_label"),
            source.source_identity(),
        ),
    };
    Ok(identity.seal()?)
}

fn aftermath_binding_identity(
    binding: &ProjectionConsumptionBindingContext,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_aftermath_binding_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("result_shape"),
                binding.result_shape_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authorized_projection_query"),
                binding.authorized_projection_query_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authorized_projection_result_shape"),
                binding.authorized_projection_result_shape_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authorized_projection"),
                binding.authorized_projection_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("narrowed_result_shape"),
                binding.narrowed_result_shape_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("policy"),
                binding.policy_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("tenant_schema_basis"),
                binding.tenant_schema_basis_digest(),
            )
            .seal()?,
    )
}

fn compose_aftermath_runtime_semantics_identity(
    runtime_semantics: &WorthQueryAftermathRuntimeSemantics,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let requested = runtime_semantics
        .requested_facts()
        .requested()
        .map(compose_fact_request_entry_digest)
        .collect::<Vec<_>>();
    Ok(domain_capability_scope_encoder(
        "worth_query_domain_capability_aftermath_runtime_semantics_v2",
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("source"),
        &aftermath_source_identity(runtime_semantics.source())?,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("binding"),
        &aftermath_binding_identity(runtime_semantics.binding())?,
    )
    .field_value_sequence(WorthQueryEvidenceTag::new("requested"), &requested)
    .seal()?)
}

fn compose_aftermath_payload_identity(
    posture: WorthQueryAftermathContributionPosture,
    semantic_code: &str,
    detail: &str,
    runtime_semantics: Option<&WorthQueryAftermathRuntimeSemantics>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_payload_v3")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            WorthQueryDomainCapabilityCategory::ConsequenceAftermath.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail);
    identity = match runtime_semantics {
        Some(runtime) => {
            let runtime_identity = runtime.semantics_identity()?;
            identity
                .field_evidence_identity(WorthQueryEvidenceTag::new("runtime"), &runtime_identity)
        }
        None => identity.field_shape(WorthQueryEvidenceTag::new("runtime"), "none"),
    };
    Ok(identity.seal()?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAftermathRuntimeSemantics {
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested_facts: ProjectMaterializedFacts,
}

impl WorthQueryAftermathRuntimeSemantics {
    pub fn new(
        source: ProjectionConsumptionSource,
        binding: ProjectionConsumptionBindingContext,
        requested_facts: ProjectMaterializedFacts,
    ) -> Self {
        Self {
            source,
            binding,
            requested_facts,
        }
    }

    pub fn source(&self) -> &ProjectionConsumptionSource {
        &self.source
    }

    pub fn binding(&self) -> &ProjectionConsumptionBindingContext {
        &self.binding
    }

    pub fn requested_facts(&self) -> &ProjectMaterializedFacts {
        &self.requested_facts
    }

    pub(in crate::domain_capabilities) fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(compose_aftermath_runtime_semantics_identity(self)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAftermathContributionPayload {
    posture: WorthQueryAftermathContributionPosture,
    semantic_code: String,
    detail: String,
    runtime_semantics: Option<WorthQueryAftermathRuntimeSemantics>,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryAftermathContributionPayload {
    pub fn new(
        posture: WorthQueryAftermathContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            posture,
            semantic_code,
            detail,
            None,
        )?)
    }

    pub fn with_runtime_semantics(
        posture: WorthQueryAftermathContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryAftermathRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity = compose_aftermath_payload_identity(
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
        WorthQueryDomainCapabilityCategory::ConsequenceAftermath
    }

    pub fn posture(&self) -> WorthQueryAftermathContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn runtime_semantics(&self) -> Option<&WorthQueryAftermathRuntimeSemantics> {
        self.runtime_semantics.as_ref()
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

impl SealedPayload for WorthQueryAftermathContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryAftermathContributionPayload {
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
