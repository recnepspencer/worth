use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::payloads::{
    WorthQueryAdmissionContributionPayload, WorthQueryAftermathContributionPayload,
    WorthQueryContinuityContributionPayload, WorthQueryDomainCapabilityCategory,
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilitySemanticPosture,
    WorthQueryExplanationContributionPayload, WorthQueryInvariantCapabilityContributionPayload,
    WorthQuerySupportContributionPayload, WorthQueryWorkflowContributionPayload,
};
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;

pub struct WorthQueryCanonicalRuntimeMaterialization<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    contribution: super::super::WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    canonical_family: &'static str,
    semantic_identity: WorthQueryEvidenceIdentity,
    materialization_identity: WorthQueryEvidenceIdentity,
}

impl<P, T> WorthQueryCanonicalRuntimeMaterialization<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub(crate) fn new(
        contribution: super::super::WorthQueryMaterializationReadyDomainCapabilityContribution<
            P,
            T,
        >,
    ) -> Result<Self, IdentityDerivationDenial> {
        let category = contribution.payload().payload().category();
        let semantic_posture = contribution.payload().payload().semantic_posture();
        let semantic_code = contribution.payload().payload().semantic_code().to_string();
        let detail = contribution.payload().payload().detail().to_string();
        let target_kind = contribution.payload().target().kind();
        let target_identity = contribution.payload().target().target_identity()?;
        let binding_identity = contribution.payload().target().binding_identity()?;
        let request_identity = contribution.payload().request_identity();
        let semantic_identity = canonical_runtime_semantic_identity(
            category,
            semantic_posture,
            &semantic_code,
            &detail,
        )?;
        let materialization_identity = domain_capability_scope_encoder(
            "worth_query_domain_capability_canonical_runtime_materialization_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            canonical_family_for(category),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_kind"),
            target_kind.as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("target"), &target_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("request"), request_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("semantic"), &semantic_identity)
        .seal()?;
        Ok(Self {
            contribution,
            canonical_family: canonical_family_for(category),
            semantic_identity,
            materialization_identity,
        })
    }

    pub fn contribution(
        &self,
    ) -> &super::super::WorthQueryMaterializationReadyDomainCapabilityContribution<P, T> {
        &self.contribution
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        self.contribution.payload().payload().category()
    }

    pub fn canonical_family(&self) -> &'static str {
        self.canonical_family
    }

    pub fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture {
        self.contribution.payload().payload().semantic_posture()
    }

    pub fn semantic_code(&self) -> &str {
        self.contribution.payload().payload().semantic_code()
    }

    pub fn detail(&self) -> &str {
        self.contribution.payload().payload().detail()
    }

    pub fn semantic_identity_for_reporting(&self) -> &str {
        self.semantic_identity.as_str()
    }

    pub fn semantic_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.semantic_identity
    }

    pub fn materialization_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.materialization_identity
    }

    pub fn materialization_digest(&self) -> &str {
        self.materialization_identity.as_str()
    }

    pub fn installed_authority(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryInstalledDomainAuthority> {
        self.contribution.payload().installed_authority()
    }

    pub fn installed_world_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.installed_authority()
            .map(crate::domain_installation::WorthQueryInstalledDomainAuthority::world_identity)
    }
}

fn canonical_runtime_semantic_identity(
    category: WorthQueryDomainCapabilityCategory,
    semantic_posture: WorthQueryDomainCapabilitySemanticPosture,
    semantic_code: &str,
    detail: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    domain_capability_scope_encoder("worth_query_domain_capability_semantic_identity_v1")
        .field_shape(WorthQueryEvidenceTag::new("category"), category.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("posture"),
            semantic_posture.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail)
        .seal()
}

pub fn materialize_domain_capability_canonical_runtime_artifact<P, T>(
    contribution: super::super::WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
) -> Result<WorthQueryCanonicalRuntimeMaterialization<P, T>, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    WorthQueryCanonicalRuntimeMaterialization::new(contribution)
}

pub type WorthQueryCanonicalAdmissionArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryAdmissionContributionPayload, T>;
pub type WorthQueryCanonicalSupportTraceabilityArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQuerySupportContributionPayload, T>;
pub type WorthQueryCanonicalInvariantCapabilityArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryInvariantCapabilityContributionPayload, T>;
pub type WorthQueryCanonicalWorkflowArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryWorkflowContributionPayload, T>;
pub type WorthQueryCanonicalContinuityArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryContinuityContributionPayload, T>;
pub type WorthQueryCanonicalAftermathArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryAftermathContributionPayload, T>;
pub type WorthQueryCanonicalExplanationArtifact<T> =
    WorthQueryCanonicalRuntimeMaterialization<WorthQueryExplanationContributionPayload, T>;

fn canonical_family_for(category: WorthQueryDomainCapabilityCategory) -> &'static str {
    match category {
        WorthQueryDomainCapabilityCategory::Admission => "intent-admission",
        WorthQueryDomainCapabilityCategory::SupportTraceability => "declaration-support",
        WorthQueryDomainCapabilityCategory::InvariantCapability => "capability-invariant",
        WorthQueryDomainCapabilityCategory::WorkflowPreview => "preview-workflow",
        WorthQueryDomainCapabilityCategory::ContinuityLineage => "continuity-lineage",
        WorthQueryDomainCapabilityCategory::ConsequenceAftermath => "consequence-aftermath",
        WorthQueryDomainCapabilityCategory::ExplanationInspection => "explanation-inspection",
    }
}
