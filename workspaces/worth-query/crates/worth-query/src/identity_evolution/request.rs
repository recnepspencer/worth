use crate::basis::{ResolvedBasisProof, ResolvedSnapshotBasis};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::{BasisDigest, CanonicalQueryDigest};
use crate::identity_authority::QueryCanonicalAuthority;

use super::families::{IdentityEvolutionQueryFamily, LineageTraversalFamily};
#[path = "request/correspondence.rs"]
mod correspondence;
pub use correspondence::{
    CorrespondenceIdentityComparison, IdentityComparisonIntent,
    IdentityEvolutionComparisonBasisFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageTraversalDescriptor {
    family: LineageTraversalFamily,
    anchor_identity: String,
    exact_result_identities: Option<Vec<String>>,
}

impl LineageTraversalDescriptor {
    pub fn direct_predecessor(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(LineageTraversalFamily::DirectPredecessor, anchor_identity)
    }

    pub fn direct_successor(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(LineageTraversalFamily::DirectSuccessor, anchor_identity)
    }

    pub fn direct_replacement(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(LineageTraversalFamily::DirectReplacement, anchor_identity)
    }

    pub fn direct_split_successors(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(
            LineageTraversalFamily::DirectSplitSuccessors,
            anchor_identity,
        )
    }

    pub fn direct_merge_successor(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(
            LineageTraversalFamily::DirectMergeSuccessor,
            anchor_identity,
        )
    }

    pub fn branch_local_direct_evolution(anchor_identity: impl Into<String>) -> Self {
        Self::from_family(
            LineageTraversalFamily::BranchLocalDirectEvolution,
            anchor_identity,
        )
    }

    pub fn family(&self) -> LineageTraversalFamily {
        self.family
    }

    pub fn anchor_identity(&self) -> &str {
        &self.anchor_identity
    }

    pub(crate) fn generated_identity(identity: impl Into<String>) -> Self {
        let identity = identity.into();
        Self::from_exact_family(
            LineageTraversalFamily::GeneratedIdentity,
            identity.clone(),
            [identity],
        )
    }

    pub(crate) fn retired_identity(identity: impl Into<String>) -> Self {
        Self::from_exact_family(
            LineageTraversalFamily::RetiredIdentity,
            identity,
            std::iter::empty(),
        )
    }

    pub(crate) fn exact_result_identities(&self) -> Option<&[String]> {
        self.exact_result_identities.as_deref()
    }

    pub(crate) fn direct_successor_exact(
        anchor_identity: impl Into<String>,
        successor_identity: impl Into<String>,
    ) -> Self {
        Self::from_exact_family(
            LineageTraversalFamily::DirectSuccessor,
            anchor_identity,
            [successor_identity.into()],
        )
    }

    pub(crate) fn direct_split_successors_exact(
        anchor_identity: impl Into<String>,
        successor_identities: impl IntoIterator<Item = String>,
    ) -> Self {
        Self::from_exact_family(
            LineageTraversalFamily::DirectSplitSuccessors,
            anchor_identity,
            successor_identities,
        )
    }

    pub(crate) fn direct_merge_successor_exact(
        anchor_identity: impl Into<String>,
        successor_identity: impl Into<String>,
    ) -> Self {
        Self::from_exact_family(
            LineageTraversalFamily::DirectMergeSuccessor,
            anchor_identity,
            [successor_identity.into()],
        )
    }

    fn from_family(family: LineageTraversalFamily, anchor_identity: impl Into<String>) -> Self {
        Self {
            family,
            anchor_identity: anchor_identity.into(),
            exact_result_identities: None,
        }
    }

    fn from_exact_family(
        family: LineageTraversalFamily,
        anchor_identity: impl Into<String>,
        result_identities: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            family,
            anchor_identity: anchor_identity.into(),
            exact_result_identities: Some(result_identities.into_iter().collect()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum IdentityEvolutionQuerySubject {
    LineageTraversal(LineageTraversalDescriptor),
    CorrespondenceIdentityComparison {
        comparison_basis_family: IdentityEvolutionComparisonBasisFamily,
        left_basis: ResolvedBasisProof,
        right_basis: ResolvedBasisProof,
        comparison: CorrespondenceIdentityComparison,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityEvolutionQueryContext {
    query_authority: QueryCanonicalAuthority,
    basis: ResolvedBasisProof,
    family: IdentityEvolutionQueryFamily,
    subject: IdentityEvolutionQuerySubject,
}

impl IdentityEvolutionQueryContext {
    pub fn query_authority(&self) -> &QueryCanonicalAuthority {
        &self.query_authority
    }

    pub fn basis_proof(&self) -> &ResolvedBasisProof {
        &self.basis
    }

    pub fn lineage_traversal(
        query_authority: &QueryCanonicalAuthority,
        basis: &ResolvedSnapshotBasis,
        descriptor: LineageTraversalDescriptor,
    ) -> Self {
        Self {
            query_authority: query_authority.clone(),
            basis: basis.proof().clone(),
            family: IdentityEvolutionQueryFamily::LineageTraversal,
            subject: IdentityEvolutionQuerySubject::LineageTraversal(descriptor),
        }
    }

    pub(crate) fn installed_operation_lineage(
        query_authority: &QueryCanonicalAuthority,
        operation_identity: &str,
        basis_capability_identity: &str,
        descriptor: LineageTraversalDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            query_authority: query_authority.clone(),
            basis: ResolvedBasisProof::from_installed_operation(
                operation_identity,
                basis_capability_identity,
            )?,
            family: IdentityEvolutionQueryFamily::LineageTraversal,
            subject: IdentityEvolutionQuerySubject::LineageTraversal(descriptor),
        })
    }

    pub(crate) fn installed_operation_correspondence(
        query_authority: &QueryCanonicalAuthority,
        operation_identity: &str,
        basis_capability_identity: &str,
        comparison: CorrespondenceIdentityComparison,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis = ResolvedBasisProof::from_installed_operation(
            operation_identity,
            basis_capability_identity,
        )?;
        Ok(Self {
            query_authority: query_authority.clone(),
            basis: basis.clone(),
            family: IdentityEvolutionQueryFamily::CorrespondenceIdentityComparison,
            subject: IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison {
                comparison_basis_family: IdentityEvolutionComparisonBasisFamily::InstalledOperation,
                left_basis: basis.clone(),
                right_basis: basis,
                comparison,
            },
        })
    }

    pub fn correspondence_identity_comparison(
        query_authority: &QueryCanonicalAuthority,
        comparison_basis_family: IdentityEvolutionComparisonBasisFamily,
        left_basis: &ResolvedSnapshotBasis,
        right_basis: &ResolvedSnapshotBasis,
        comparison: CorrespondenceIdentityComparison,
    ) -> Self {
        Self {
            query_authority: query_authority.clone(),
            basis: left_basis.proof().clone(),
            family: IdentityEvolutionQueryFamily::CorrespondenceIdentityComparison,
            subject: IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison {
                comparison_basis_family,
                left_basis: left_basis.proof().clone(),
                right_basis: right_basis.proof().clone(),
                comparison,
            },
        }
    }

    pub fn query_digest(&self) -> &CanonicalQueryDigest {
        self.query_authority.digest()
    }

    #[cfg(test)]
    pub(crate) fn lineage_traversal_for_test(
        query_digest: CanonicalQueryDigest,
        basis_digest: BasisDigest,
        descriptor: LineageTraversalDescriptor,
    ) -> Self {
        Self {
            query_authority: canonical_authority_for_test(&query_digest),
            basis: ResolvedBasisProof::from_digest_for_test(basis_digest),
            family: IdentityEvolutionQueryFamily::LineageTraversal,
            subject: IdentityEvolutionQuerySubject::LineageTraversal(descriptor),
        }
    }

    #[cfg(test)]
    pub(crate) fn correspondence_identity_comparison_for_test(
        query_digest: CanonicalQueryDigest,
        comparison_basis_family: IdentityEvolutionComparisonBasisFamily,
        left_basis_digest: BasisDigest,
        right_basis_digest: BasisDigest,
        comparison: CorrespondenceIdentityComparison,
    ) -> Self {
        Self {
            query_authority: canonical_authority_for_test(&query_digest),
            basis: ResolvedBasisProof::from_digest_for_test(left_basis_digest.clone()),
            family: IdentityEvolutionQueryFamily::CorrespondenceIdentityComparison,
            subject: IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison {
                comparison_basis_family,
                left_basis: ResolvedBasisProof::from_digest_for_test(left_basis_digest),
                right_basis: ResolvedBasisProof::from_digest_for_test(right_basis_digest),
                comparison,
            },
        }
    }

    pub fn basis_digest(&self) -> &BasisDigest {
        self.basis.digest()
    }

    pub fn family(&self) -> IdentityEvolutionQueryFamily {
        self.family
    }

    pub fn lineage_traversal_descriptor(&self) -> Option<&LineageTraversalDescriptor> {
        match &self.subject {
            IdentityEvolutionQuerySubject::LineageTraversal(descriptor) => Some(descriptor),
            IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison { .. } => None,
        }
    }

    pub fn correspondence_identity_comparison_descriptor(
        &self,
    ) -> Option<(
        IdentityEvolutionComparisonBasisFamily,
        &BasisDigest,
        &BasisDigest,
        &CorrespondenceIdentityComparison,
    )> {
        match &self.subject {
            IdentityEvolutionQuerySubject::LineageTraversal(_) => None,
            IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison {
                comparison_basis_family,
                left_basis,
                right_basis,
                comparison,
            } => Some((
                *comparison_basis_family,
                left_basis.digest(),
                right_basis.digest(),
                comparison,
            )),
        }
    }

    pub fn correspondence_basis_proofs(
        &self,
    ) -> Option<(&ResolvedBasisProof, &ResolvedBasisProof)> {
        match &self.subject {
            IdentityEvolutionQuerySubject::LineageTraversal(_) => None,
            IdentityEvolutionQuerySubject::CorrespondenceIdentityComparison {
                left_basis,
                right_basis,
                ..
            } => Some((left_basis, right_basis)),
        }
    }
}

#[cfg(test)]
pub(super) fn canonical_authority_for_test(seed: &CanonicalQueryDigest) -> QueryCanonicalAuthority {
    use worth_query_declaration::facade::authoring::{
        AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
        RawAuthoredResultShape, RootEntityKey,
    };
    use worth_query_declaration::facade::canonicalization::canonicalize_request;

    let root = RootEntityKey::new(format!("test_{}", seed.as_str())).unwrap();
    let query = RawAuthoredQuery::detail_builder(root)
        .project(AspectFieldSelector::new("value", "text").unwrap())
        .build()
        .unwrap();
    let shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("value", "text", "value").unwrap())
        .build()
        .unwrap();
    canonicalize_request(GuidedAuthoringPath::pair_detail(query, shape).unwrap())
        .unwrap()
        .query()
        .authority()
        .clone()
}
