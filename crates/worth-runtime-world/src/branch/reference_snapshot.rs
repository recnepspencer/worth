use std::sync::Arc;

use crate::history::CompositeRuntimeWorldCommit;
use crate::identity::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration, RuntimeWorldOwnerIdentity,
};

/// One owner-issued immutable product-reference image. Its commit supplies
/// both selected identity and basis, preventing mixed observations. The value
/// is evidence captured at one linearization point, not continuing authority
/// or a promise that the referenced head remains current.
#[derive(Debug, Clone)]
pub struct ProductBranchReferenceSnapshot {
    owner: RuntimeWorldOwnerIdentity,
    branch: ProductBranchIdentity,
    lifecycle: ProductBranchIncarnation,
    generation: ProductBranchReferenceGeneration,
    commit: Arc<CompositeRuntimeWorldCommit>,
}

impl PartialEq for ProductBranchReferenceSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.branch == other.branch
            && self.lifecycle == other.lifecycle
            && self.generation == other.generation
            && self.commit.identity() == other.commit.identity()
            && crate::basis::compare_exact(self.commit.basis(), other.commit.basis()).is_ok()
    }
}

impl Eq for ProductBranchReferenceSnapshot {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBranchReferenceSnapshotDenial {
    BranchOwnerMismatch,
    LifecycleOwnerMismatch,
    CommitOwnerMismatch,
    BasisOwnerMismatch,
}

impl ProductBranchReferenceSnapshot {
    pub(crate) fn owner_issued(
        owner: RuntimeWorldOwnerIdentity,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        generation: ProductBranchReferenceGeneration,
        commit: Arc<CompositeRuntimeWorldCommit>,
    ) -> Result<Self, ProductBranchReferenceSnapshotDenial> {
        if branch.owner_identity() != owner {
            return Err(ProductBranchReferenceSnapshotDenial::BranchOwnerMismatch);
        }
        if lifecycle.owner_identity() != owner {
            return Err(ProductBranchReferenceSnapshotDenial::LifecycleOwnerMismatch);
        }
        if commit.identity().owner_identity() != owner {
            return Err(ProductBranchReferenceSnapshotDenial::CommitOwnerMismatch);
        }
        if commit.basis().owner_identity() != owner {
            return Err(ProductBranchReferenceSnapshotDenial::BasisOwnerMismatch);
        }
        Ok(Self {
            owner,
            branch,
            lifecycle,
            generation,
            commit,
        })
    }

    pub(crate) const fn owner(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub(crate) fn branch(&self) -> &ProductBranchIdentity {
        &self.branch
    }

    pub(crate) const fn lifecycle(&self) -> ProductBranchIncarnation {
        self.lifecycle
    }

    pub(crate) const fn generation(&self) -> ProductBranchReferenceGeneration {
        self.generation
    }

    pub(crate) fn commit(&self) -> &CompositeRuntimeWorldCommit {
        &self.commit
    }

    /// The commit this image was captured at, shared: a reuse of this head
    /// installs the very commit the image names rather than looking it up.
    pub(crate) fn shared_commit(&self) -> Arc<CompositeRuntimeWorldCommit> {
        Arc::clone(&self.commit)
    }

    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        &self.branch
    }

    pub fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.lifecycle
    }

    pub fn reference_generation(&self) -> ProductBranchReferenceGeneration {
        self.generation
    }

    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        self.commit.identity()
    }

    pub fn basis(&self) -> &crate::basis::AdmittedCompositeRuntimeWorldBasis {
        self.commit.basis()
    }
}
