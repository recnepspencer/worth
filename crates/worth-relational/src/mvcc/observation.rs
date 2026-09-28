use std::sync::Arc;

use crate::branch::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisInner,
    RelationalBranchBasisDescriptor, RelationalBranchIdentity,
    RelationalBranchReferenceObservation, RelationalBranchRoot, RelationalBranchVersion,
};

/// Immutable repeatable read view issued from one admitted branch basis.
///
/// The observation shares the admitted root and lease. It performs no live
/// branch lookup, and therefore remains stable when the branch reference later
/// moves.
#[derive(Clone, Debug)]
pub struct RelationalBranchObservation {
    inner: Arc<AdmittedRelationalBranchBasisInner>,
}

impl AdmittedRelationalBranchBasis {
    pub fn observation(&self) -> RelationalBranchObservation {
        RelationalBranchObservation {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl RelationalBranchObservation {
    pub(crate) fn admitted_basis(&self) -> AdmittedRelationalBranchBasis {
        AdmittedRelationalBranchBasis {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Whether this observation was issued from `basis`.
    pub fn issued_from(&self, basis: &AdmittedRelationalBranchBasis) -> bool {
        self.admitted_basis() == *basis
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.inner.descriptor
    }

    pub fn identity(&self) -> &RelationalBranchIdentity {
        &self.inner.identity
    }

    pub fn reference(&self) -> &RelationalBranchReferenceObservation {
        self.inner.descriptor.reference()
    }

    pub fn truth_version(&self) -> RelationalBranchVersion {
        self.inner.descriptor.truth_version()
    }

    /// Canonical storage version selected by this exact branch observation.
    pub fn version_id(&self) -> crate::identity::data::VersionId {
        match self.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                crate::identity::data::VersionId(0)
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                crate::identity::data::VersionId(target.version_id())
            }
        }
    }

    /// Canonical commit selected by the branch reference. Metadata-only
    /// commits can advance this axis while retaining the same truth root.
    pub fn commit_id(&self) -> Option<crate::history::data::CommitId> {
        match self.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => None,
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                Some(crate::history::data::CommitId(target.selected_commit_id()))
            }
        }
    }

    /// Canonical commit selected by this immutable observation. The commit is
    /// reached from the same root as storage and visibility, so it is valid
    /// immediately at branch-reference movement without a catalog refresh.
    pub fn canonical_commit(&self) -> Option<&crate::history::data::CanonicalCommitEnvelope> {
        self.inner
            .root
            .canonical_envelope()
            .map(std::sync::Arc::as_ref)
    }

    pub fn commit_receipt(&self) -> Option<&crate::history::data::RelationalCommitReceipt> {
        self.canonical_commit().map(|envelope| &envelope.commit)
    }

    pub fn canonical_patch(
        &self,
    ) -> Option<&crate::publication::patch::data::CanonicalAuthoritativePatch> {
        self.canonical_commit().map(|envelope| &envelope.patch)
    }

    pub fn correctness_index(&self) -> Option<crate::branch::RelationalRootCorrectnessIndex> {
        self.inner.root.axes().map(|axes| axes.correctness_index)
    }

    pub fn selected_root_identity(&self) -> u64 {
        self.inner.root.id()
    }

    pub(crate) fn selected_root(&self) -> &Arc<RelationalBranchRoot> {
        &self.inner.root
    }
}
