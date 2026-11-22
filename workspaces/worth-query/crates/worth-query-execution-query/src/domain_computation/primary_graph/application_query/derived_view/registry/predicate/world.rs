//! Exact World branch occurrence; Native branch names are insufficient.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PredicateWorld {
    pub branch: worth_runtime_world::facade::ProductBranchIdentity,
    pub incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
}
impl PredicateWorld {
    pub fn of(observation: &worth_runtime_world::facade::ProductBranchObservation) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
        }
    }
    pub fn name_bytes(&self) -> usize {
        self.branch.name().as_str().len()
    }
}
