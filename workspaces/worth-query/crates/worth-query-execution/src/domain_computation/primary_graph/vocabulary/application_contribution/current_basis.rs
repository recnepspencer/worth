//! Moved whole from `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::current_basis`: the items below and every impl they own.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentBasis {
    commit: worth_runtime_world::facade::CompositeCommitIdentity,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl CurrentBasis {
    pub fn of(observation: &worth_runtime_world::facade::ProductBranchObservation) -> Self {
        Self {
            commit: observation.selected_commit().clone(),
            occurrence: observation.lifecycle_incarnation(),
        }
    }
}
