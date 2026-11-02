//! Exact World branch occurrence; Native branch names are insufficient.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct PredicateWorld {
    pub(super) branch: worth_runtime_world::facade::ProductBranchIdentity,
    pub(super) incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
}
impl PredicateWorld {
    pub(in crate::domain_computation::primary_graph) fn of(
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
        }
    }
    pub(super) fn name_bytes(&self) -> usize {
        self.branch.name().as_str().len()
    }
}
