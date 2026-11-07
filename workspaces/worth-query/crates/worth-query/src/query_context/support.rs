use super::basis::{ComparisonBasisFamily, QueryContextFamily};
use super::identity::compose_query_context_support_profile_digest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryContextDeferredScopeMarker {
    StoreBackedHistorical,
    StoreBackedDiff,
    BroadCollectionDiff,
}

impl QueryContextDeferredScopeMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StoreBackedHistorical => "store_backed_historical",
            Self::StoreBackedDiff => "store_backed_diff",
            Self::BroadCollectionDiff => "broad_collection_diff",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryContextSupportProfile {
    admitted_basis_families: Vec<QueryContextFamily>,
    admitted_comparison_families: Vec<ComparisonBasisFamily>,
    deferred_scope_markers: Vec<QueryContextDeferredScopeMarker>,
    profile_digest: String,
}

impl QueryContextSupportProfile {
    pub fn admitted_basis_families(&self) -> &[QueryContextFamily] {
        &self.admitted_basis_families
    }

    pub fn admitted_comparison_families(&self) -> &[ComparisonBasisFamily] {
        &self.admitted_comparison_families
    }

    pub fn deferred_scope_markers(&self) -> &[QueryContextDeferredScopeMarker] {
        &self.deferred_scope_markers
    }

    pub fn profile_digest(&self) -> &str {
        &self.profile_digest
    }
}

pub fn runtime_backed_narrow_query_context_support_profile(
) -> Result<QueryContextSupportProfile, IdentityDerivationDenial> {
    let admitted_basis_families = vec![
        QueryContextFamily::CurrentBranchHead,
        QueryContextFamily::BranchHead,
        QueryContextFamily::HistoricalSnapshot,
        QueryContextFamily::HistoricalCommit,
        QueryContextFamily::PreviewDerivedHistorical,
    ];
    let admitted_comparison_families = vec![
        ComparisonBasisFamily::BranchToBranch,
        ComparisonBasisFamily::CurrentToHistorical,
        ComparisonBasisFamily::HistoricalToHistorical,
        ComparisonBasisFamily::PreviewToAuthoritative,
    ];
    let deferred_scope_markers = vec![
        QueryContextDeferredScopeMarker::StoreBackedHistorical,
        QueryContextDeferredScopeMarker::StoreBackedDiff,
        QueryContextDeferredScopeMarker::BroadCollectionDiff,
    ];
    let profile_digest = compose_query_context_support_profile_digest(
        &admitted_basis_families,
        &admitted_comparison_families,
        &deferred_scope_markers,
    )?;

    Ok(QueryContextSupportProfile {
        admitted_basis_families,
        admitted_comparison_families,
        deferred_scope_markers,
        profile_digest,
    })
}
