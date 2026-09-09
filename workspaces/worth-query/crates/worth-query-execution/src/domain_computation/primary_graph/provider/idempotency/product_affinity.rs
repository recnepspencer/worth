use worth_runtime_world::facade::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchObservation,
    ProductBranchReferenceSnapshot,
};

/// Descriptive cache affinity for one World product-branch occurrence.
///
/// Relational component branches may be shared by multiple products. They are
/// therefore insufficient to index a completed or unpublished product result.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryProductIdempotencyAffinity {
    branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
}

impl WorthQueryProductIdempotencyAffinity {
    pub(in crate::domain_computation::primary_graph) fn from_observation(
        observation: &ProductBranchObservation,
    ) -> Self {
        Self::new(
            observation.branch_identity().clone(),
            observation.lifecycle_incarnation(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn from_reference(
        snapshot: &ProductBranchReferenceSnapshot,
    ) -> Self {
        Self::new(
            snapshot.branch_identity().clone(),
            snapshot.lifecycle_incarnation(),
        )
    }

    fn new(branch: ProductBranchIdentity, incarnation: ProductBranchIncarnation) -> Self {
        Self {
            branch,
            incarnation,
        }
    }
}
