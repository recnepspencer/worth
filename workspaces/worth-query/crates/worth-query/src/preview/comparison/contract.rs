use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::{
    CanonicalQueryDigest, CanonicalResultShapeDigest, CollectionPlanDigest, ResultDigest,
    ValidatedQueryDigest,
};
use crate::preview::execution::PreviewComparisonCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewComparisonEligibilityArtifact {
    pub(super) digest: String,
    pub(in crate::preview) canonical_query_digest: CanonicalQueryDigest,
    pub(in crate::preview) canonical_result_shape_digest: CanonicalResultShapeDigest,
    pub(super) collection_digest: Option<CollectionPlanDigest>,
    pub(super) result_family: String,
    pub(super) ordering_digest: String,
    pub(super) materialization_boundary_digest: String,
    pub(super) shape_check_width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct PreviewComparisonCandidateArtifact {
    pub(super) digest: String,
    pub(super) validated_query_digest: ValidatedQueryDigest,
    pub(super) basis_digest: String,
    pub(super) result_digest: ResultDigest,
    pub(super) canonical_query_digest: CanonicalQueryDigest,
    pub(super) canonical_result_shape_digest: CanonicalResultShapeDigest,
    pub(super) collection_digest: Option<CollectionPlanDigest>,
    pub(super) result_family: String,
    pub(super) ordering_digest: String,
    pub(super) materialization_boundary_digest: String,
    pub(super) shape_check_width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativePreviewComparisonCandidate {
    pub(super) artifact: PreviewComparisonCandidateArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewComparisonFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    CandidateBasisAuthorityMismatch,
    CandidateExecutionPlanMismatch,
    CandidateExecutionBasisMismatch,
    QueryDigestMismatch,
    ResultShapeMismatch,
    ResultFamilyMismatch,
    OrderingBasisMismatch,
    MaterializationBoundaryMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewComparisonError {
    pub(super) failure_class: PreviewComparisonFailureClass,
    pub(super) message: &'static str,
    pub(super) preview_digest: String,
    pub(super) candidate_digest: String,
    pub(super) counters: PreviewComparisonCounters,
}

impl PreviewComparisonError {
    pub(super) fn identity_refused(
        cause: IdentityDerivationDenial,
        preview: Option<&str>,
        candidate: Option<&str>,
        counters: PreviewComparisonCounters,
    ) -> Self {
        Self {
            failure_class: PreviewComparisonFailureClass::EvidenceIdentityDerivationDenied(cause),
            message: "preview comparison evidence identity refused",
            preview_digest: preview.unwrap_or_default().to_owned(),
            candidate_digest: candidate.unwrap_or_default().to_owned(),
            counters,
        }
    }

    pub fn failure_class(&self) -> &PreviewComparisonFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn preview_digest(&self) -> &str {
        &self.preview_digest
    }

    pub fn candidate_digest(&self) -> &str {
        &self.candidate_digest
    }

    pub fn counters(&self) -> &PreviewComparisonCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewExecutionComparisonAdmission {
    pub(super) digest: String,
    pub(super) preview_execution_digest: String,
    pub(super) preview_comparison_digest: String,
    pub(super) candidate_comparison_digest: String,
    pub(super) canonical_query_digest: CanonicalQueryDigest,
    pub(super) validated_query_digest: ValidatedQueryDigest,
    pub(super) candidate_basis_digest: String,
    pub(super) candidate_result_digest: ResultDigest,
    pub(super) shape_check_width: usize,
    pub(super) counters: PreviewComparisonCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionParityPreviewComparisonAdmission {
    pub(super) inner: PreviewExecutionComparisonAdmission,
}

impl PreviewComparisonEligibilityArtifact {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        &self.canonical_query_digest
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        &self.canonical_result_shape_digest
    }

    pub fn collection_digest(&self) -> Option<&CollectionPlanDigest> {
        self.collection_digest.as_ref()
    }

    pub fn result_family(&self) -> &str {
        &self.result_family
    }

    pub fn ordering_digest(&self) -> &str {
        &self.ordering_digest
    }

    pub fn materialization_boundary_digest(&self) -> &str {
        &self.materialization_boundary_digest
    }

    pub fn shape_check_width(&self) -> usize {
        self.shape_check_width
    }
}

impl PreviewComparisonCandidateArtifact {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        &self.validated_query_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn result_digest(&self) -> &ResultDigest {
        &self.result_digest
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        &self.canonical_query_digest
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        &self.canonical_result_shape_digest
    }

    pub fn collection_digest(&self) -> Option<&CollectionPlanDigest> {
        self.collection_digest.as_ref()
    }

    pub fn result_family(&self) -> &str {
        &self.result_family
    }

    pub fn ordering_digest(&self) -> &str {
        &self.ordering_digest
    }

    pub fn materialization_boundary_digest(&self) -> &str {
        &self.materialization_boundary_digest
    }

    pub fn shape_check_width(&self) -> usize {
        self.shape_check_width
    }
}

impl AuthoritativePreviewComparisonCandidate {
    pub fn digest(&self) -> &str {
        self.artifact.digest()
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        self.artifact.validated_query_digest()
    }

    pub fn basis_digest(&self) -> &str {
        self.artifact.basis_digest()
    }

    pub fn result_digest(&self) -> &ResultDigest {
        self.artifact.result_digest()
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        self.artifact.canonical_query_digest()
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        self.artifact.canonical_result_shape_digest()
    }

    pub fn collection_digest(&self) -> Option<&CollectionPlanDigest> {
        self.artifact.collection_digest()
    }

    pub fn result_family(&self) -> &str {
        self.artifact.result_family()
    }

    pub fn ordering_digest(&self) -> &str {
        self.artifact.ordering_digest()
    }

    pub fn materialization_boundary_digest(&self) -> &str {
        self.artifact.materialization_boundary_digest()
    }

    pub fn shape_check_width(&self) -> usize {
        self.artifact.shape_check_width()
    }

    pub(crate) fn artifact(&self) -> &PreviewComparisonCandidateArtifact {
        &self.artifact
    }
}

impl PreviewExecutionComparisonAdmission {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn preview_execution_digest(&self) -> &str {
        &self.preview_execution_digest
    }

    pub fn preview_comparison_digest(&self) -> &str {
        &self.preview_comparison_digest
    }

    pub fn candidate_comparison_digest(&self) -> &str {
        &self.candidate_comparison_digest
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        &self.canonical_query_digest
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        &self.validated_query_digest
    }

    pub fn candidate_basis_digest(&self) -> &str {
        &self.candidate_basis_digest
    }

    pub fn candidate_result_digest(&self) -> &ResultDigest {
        &self.candidate_result_digest
    }

    pub fn shape_check_width(&self) -> usize {
        self.shape_check_width
    }

    pub fn counters(&self) -> &PreviewComparisonCounters {
        &self.counters
    }
}

impl PromotionParityPreviewComparisonAdmission {
    pub fn digest(&self) -> &str {
        self.inner.digest()
    }

    pub fn preview_execution_digest(&self) -> &str {
        self.inner.preview_execution_digest()
    }

    pub fn preview_comparison_digest(&self) -> &str {
        self.inner.preview_comparison_digest()
    }

    pub fn candidate_comparison_digest(&self) -> &str {
        self.inner.candidate_comparison_digest()
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        self.inner.canonical_query_digest()
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        self.inner.validated_query_digest()
    }

    pub fn candidate_basis_digest(&self) -> &str {
        self.inner.candidate_basis_digest()
    }

    pub fn candidate_result_digest(&self) -> &ResultDigest {
        self.inner.candidate_result_digest()
    }

    pub fn shape_check_width(&self) -> usize {
        self.inner.shape_check_width()
    }

    pub fn counters(&self) -> &PreviewComparisonCounters {
        self.inner.counters()
    }

    #[cfg(test)]
    pub(crate) fn as_preview_comparison(&self) -> &PreviewExecutionComparisonAdmission {
        &self.inner
    }
}
