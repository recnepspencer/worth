use crate::correspondence::{
    CorrespondenceEvaluationRequest, StructuralCandidateBudget, StructuralCandidateDiscoveryPlan,
    StructuralCandidateOrderingContract,
};
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryContinuityCorrespondenceSemantics {
    LineageOnly {
        canonical_subject: String,
        authoritative_counterpart: String,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: StructuralCandidateBudget,
    },
    StructuralOnly {
        candidates: Vec<String>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: StructuralCandidateBudget,
        ordering_contract: StructuralCandidateOrderingContract,
    },
    Mixed {
        canonical_subject: String,
        authoritative_counterpart: String,
        candidates: Vec<String>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: StructuralCandidateBudget,
        ordering_contract: StructuralCandidateOrderingContract,
    },
}

impl WorthQueryContinuityCorrespondenceSemantics {
    pub fn lineage_only(
        canonical_subject: impl Into<String>,
        authoritative_counterpart: impl Into<String>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
    ) -> Self {
        Self::LineageOnly {
            canonical_subject: canonical_subject.into(),
            authoritative_counterpart: authoritative_counterpart.into(),
            discovery_plan,
            budget: StructuralCandidateBudget::bounded(budget),
        }
    }

    pub fn structural_only(
        candidates: impl IntoIterator<Item = impl Into<String>>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
        ordering_contract: StructuralCandidateOrderingContract,
    ) -> Self {
        Self::StructuralOnly {
            candidates: candidates.into_iter().map(Into::into).collect(),
            discovery_plan,
            budget: StructuralCandidateBudget::bounded(budget),
            ordering_contract,
        }
    }

    pub fn mixed(
        canonical_subject: impl Into<String>,
        authoritative_counterpart: impl Into<String>,
        candidates: impl IntoIterator<Item = impl Into<String>>,
        discovery_plan: StructuralCandidateDiscoveryPlan,
        budget: usize,
        ordering_contract: StructuralCandidateOrderingContract,
    ) -> Self {
        Self::Mixed {
            canonical_subject: canonical_subject.into(),
            authoritative_counterpart: authoritative_counterpart.into(),
            candidates: candidates.into_iter().map(Into::into).collect(),
            discovery_plan,
            budget: StructuralCandidateBudget::bounded(budget),
            ordering_contract,
        }
    }

    pub fn to_request(&self) -> CorrespondenceEvaluationRequest {
        match self {
            Self::LineageOnly {
                canonical_subject,
                authoritative_counterpart,
                discovery_plan,
                budget,
            } => CorrespondenceEvaluationRequest::lineage_only(
                canonical_subject.clone(),
                authoritative_counterpart.clone(),
                discovery_plan.clone(),
                budget.max_candidates(),
            ),
            Self::StructuralOnly {
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            } => CorrespondenceEvaluationRequest::structural_only(
                candidates.clone(),
                discovery_plan.clone(),
                budget.max_candidates(),
                ordering_contract.clone(),
            ),
            Self::Mixed {
                canonical_subject,
                authoritative_counterpart,
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            } => CorrespondenceEvaluationRequest::mixed(
                canonical_subject.clone(),
                authoritative_counterpart.clone(),
                candidates.clone(),
                discovery_plan.clone(),
                budget.max_candidates(),
                ordering_contract.clone(),
            ),
        }
    }

    pub fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        let mut identity =
            domain_capability_scope_encoder("worth_query_continuity_correspondence_semantics_v1");
        identity = match self {
            Self::LineageOnly {
                canonical_subject,
                authoritative_counterpart,
                discovery_plan,
                budget,
            } => identity
                .field_shape(WorthQueryEvidenceTag::new("kind"), "lineage")
                .field_shape(
                    WorthQueryEvidenceTag::new("canonical_subject"),
                    canonical_subject,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("authoritative_counterpart"),
                    authoritative_counterpart,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("discovery_plan"),
                    discovery_plan.as_str(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("budget"),
                    budget.max_candidates(),
                ),
            Self::StructuralOnly {
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            } => identity
                .field_shape(WorthQueryEvidenceTag::new("kind"), "structural")
                .field_value_sequence(WorthQueryEvidenceTag::new("candidates"), candidates)
                .field_shape(
                    WorthQueryEvidenceTag::new("discovery_plan"),
                    discovery_plan.as_str(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("budget"),
                    budget.max_candidates(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("ordering_contract"),
                    ordering_contract.as_str(),
                ),
            Self::Mixed {
                canonical_subject,
                authoritative_counterpart,
                candidates,
                discovery_plan,
                budget,
                ordering_contract,
            } => identity
                .field_shape(WorthQueryEvidenceTag::new("kind"), "mixed")
                .field_shape(
                    WorthQueryEvidenceTag::new("canonical_subject"),
                    canonical_subject,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("authoritative_counterpart"),
                    authoritative_counterpart,
                )
                .field_value_sequence(WorthQueryEvidenceTag::new("candidates"), candidates)
                .field_shape(
                    WorthQueryEvidenceTag::new("discovery_plan"),
                    discovery_plan.as_str(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("budget"),
                    budget.max_candidates(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("ordering_contract"),
                    ordering_contract.as_str(),
                ),
        };
        Ok(identity.seal()?)
    }

    pub fn digest_fragment(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.semantics_identity()?.as_str().to_string())
    }
}
