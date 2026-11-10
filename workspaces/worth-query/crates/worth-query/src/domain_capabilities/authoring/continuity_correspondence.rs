use crate::correspondence::{
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use crate::domain_capabilities::payloads::{
    WorthQueryContinuityContributionPayload, WorthQueryContinuityContributionPosture,
    WorthQueryContinuityCorrespondenceSemantics,
};

use super::continuity::WorthQueryContinuityContributionAuthoring;

impl WorthQueryContinuityContributionAuthoring {
    pub fn correspondence_lineage_only(
        canonical_subject: impl Into<String>,
        authoritative_counterpart: impl Into<String>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_correspondence_semantics(
            WorthQueryContinuityContributionPosture::CorrespondenceOnly,
            semantic_code,
            detail,
            WorthQueryContinuityCorrespondenceSemantics::lineage_only(
                canonical_subject,
                authoritative_counterpart,
                discovery_plan,
                budget,
            ),
        )
    }

    pub fn correspondence_structural_only(
        candidates: impl IntoIterator<Item = impl Into<String>>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
        ordering_contract: StructuralCandidateOrderingContract,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_correspondence_semantics(
            WorthQueryContinuityContributionPosture::CorrespondenceOnly,
            semantic_code,
            detail,
            WorthQueryContinuityCorrespondenceSemantics::structural_only(
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            ),
        )
    }

    pub fn correspondence_mixed(
        canonical_subject: impl Into<String>,
        authoritative_counterpart: impl Into<String>,
        candidates: impl IntoIterator<Item = impl Into<String>>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
        ordering_contract: StructuralCandidateOrderingContract,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_correspondence_semantics(
            WorthQueryContinuityContributionPosture::CorrespondenceOnly,
            semantic_code,
            detail,
            WorthQueryContinuityCorrespondenceSemantics::mixed(
                canonical_subject,
                authoritative_counterpart,
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            ),
        )
    }

    fn with_correspondence_semantics(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        correspondence_semantics: WorthQueryContinuityCorrespondenceSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryContinuityContributionPayload::with_correspondence_semantics(
                posture,
                semantic_code,
                detail,
                Some(correspondence_semantics),
            )?,
        })
    }
}
