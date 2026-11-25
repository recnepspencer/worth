use std::sync::Arc;
pub mod retained_retirement;
pub mod retirement;
pub use retained_retirement::RelationalRetainedObservationShares;
pub use retirement::RelationalObservationRetirement;

use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::branch::basis::SharedAdmittedRelationalBranchBasisInner;
use crate::branch::identity::RelationalBranchIdentity;
use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::root::RelationalBranchRoot;
use crate::branch::version::RelationalBranchVersion;

/// Immutable repeatable read view issued from one admitted branch basis.
///
/// The observation shares the admitted root and lease. It performs no live
/// branch lookup, and therefore remains stable when the branch reference later
/// moves.
#[derive(Clone, Debug)]
pub struct RelationalBranchObservation {
    inner: SharedAdmittedRelationalBranchBasisInner,
}

impl AdmittedRelationalBranchBasis {
    pub(crate) fn into_observation(self) -> RelationalBranchObservation {
        RelationalBranchObservation { inner: self.inner }
    }
    pub fn observation(&self) -> RelationalBranchObservation {
        RelationalBranchObservation {
            inner: self.inner.clone(),
        }
    }
}

impl RelationalBranchObservation {
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
        self.inner.root.commit_receipt()
    }

    pub fn canonical_patch(
        &self,
    ) -> Option<&crate::publication::patch::data::CanonicalAuthoritativePatch> {
        self.canonical_commit().map(|envelope| &envelope.patch)
    }

    pub fn correctness_index(&self) -> Option<crate::branch::root::RelationalRootCorrectnessIndex> {
        self.inner.root.axes().map(|axes| axes.correctness_index)
    }

    pub fn selected_root_identity(&self) -> u64 {
        self.inner.root.id()
    }
}

impl RelationalBranchObservation {
    /// Whether the schema retained with this observation declares `aspect`
    /// for entity kind `kind_id`. A later schema change never alters the
    /// answer.
    pub fn entity_kind_declares_aspect(
        &self,
        kind_id: crate::identity::data::KindId,
        aspect: &worth_foundational::facade::AspectKey,
    ) -> bool {
        self.selected_root()
            .schema_authority()
            .entity_aspect_plan(kind_id)
            .and_then(|plan| plan.contract_for(aspect))
            .is_some()
    }

    /// Whether the schema retained with this observation declares `aspect`
    /// for relation kind `kind_id`. A later schema change never alters the
    /// answer.
    pub fn relation_kind_declares_aspect(
        &self,
        kind_id: crate::identity::data::KindId,
        aspect: &worth_foundational::facade::AspectKey,
    ) -> bool {
        self.selected_root()
            .schema_authority()
            .relation_aspect_plan(kind_id)
            .and_then(|plan| plan.contract_for(aspect))
            .is_some()
    }
}

/// Operations used by the Relational owners.
pub trait RelationalBranchObservationOwnerPort: Sized {
    fn admitted_basis(&self) -> AdmittedRelationalBranchBasis;
    fn selected_root(&self) -> &Arc<RelationalBranchRoot>;
}

impl RelationalBranchObservationOwnerPort for RelationalBranchObservation {
    fn admitted_basis(&self) -> AdmittedRelationalBranchBasis {
        AdmittedRelationalBranchBasis::from_inner(self.inner.clone())
    }

    fn selected_root(&self) -> &Arc<RelationalBranchRoot> {
        &self.inner.root
    }
}
