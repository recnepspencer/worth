use worth_runtime_world::facade::{
    CompositeBasisKey, CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration,
};

/// Describes the exact World occurrence used by a read. It carries no live
/// admission or publication authority and retains no owner resources.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProductBranchReadIdentity {
    branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
    generation: ProductBranchReferenceGeneration,
    commit: CompositeCommitIdentity,
    basis: CompositeBasisKey,
}

impl WorthQueryProductBranchReadIdentity {
    pub(crate) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
            generation: observation.reference_generation(),
            commit: observation.selected_commit().clone(),
            basis: observation.basis().identity().clone(),
        }
    }

    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        &self.branch
    }
    pub fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.incarnation
    }
    pub fn reference_generation(&self) -> ProductBranchReferenceGeneration {
        self.generation
    }
    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        &self.commit
    }
    pub fn composite_basis_identity(&self) -> &CompositeBasisKey {
        &self.basis
    }
}
