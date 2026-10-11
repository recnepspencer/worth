//! An immutable Product commit determines its component basis; incarnation
//! distinguishes retire/recreate. Both owner-issued axes have fixed-size copies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::domain_computation::primary_graph) struct CurrentBasis {
    commit: worth_runtime_world::facade::CompositeCommitIdentity,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
}
impl CurrentBasis {
    pub(in crate::domain_computation::primary_graph) fn of(
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Self {
        Self {
            commit: observation.selected_commit().clone(),
            occurrence: observation.lifecycle_incarnation(),
        }
    }
}
