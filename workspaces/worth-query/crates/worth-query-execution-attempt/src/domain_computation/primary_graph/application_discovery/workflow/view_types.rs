//! Pure workflow history view wrapper structs.
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use std::sync::Arc;
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowRecordedActor(pub(crate) EntityId);

impl WorthQueryWorkflowRecordedActor {
    pub(crate) fn observed(entity: EntityId) -> Self {
        Self(entity)
    }

    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

#[derive(Clone)]
pub struct WorthQueryWorkflowReadBasis(pub(crate) Arc<WorthQueryApplicationReadObservation>);

impl WorthQueryWorkflowReadBasis {
    pub(crate) fn from_observation(observation: Arc<WorthQueryApplicationReadObservation>) -> Self {
        Self(observation)
    }

    pub(crate) fn observation(&self) -> &WorthQueryApplicationReadObservation {
        &self.0
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.0.selected_commit()
    }
}

impl WorthQueryWorkflowReadBasis {
    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.0.branch_identity()
    }
}
