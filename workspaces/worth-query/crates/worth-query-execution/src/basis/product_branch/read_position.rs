use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration,
};

/// Declared World occurrence axes used to order conditional evaluations.
/// The owner-issued immutable commit determines its basis; the history owner
/// refuses duplicate commits. Exact admission tokens add no ordering axis.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct WorthQueryProductBranchReadPosition {
    pub(super) branch: ProductBranchIdentity,
    pub(super) incarnation: ProductBranchIncarnation,
    pub(super) generation: ProductBranchReferenceGeneration,
    pub(super) commit: CompositeCommitIdentity,
}

impl WorthQueryProductBranchReadPosition {
    pub(super) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
            generation: observation.reference_generation(),
            commit: observation.selected_commit().clone(),
        }
    }
}
