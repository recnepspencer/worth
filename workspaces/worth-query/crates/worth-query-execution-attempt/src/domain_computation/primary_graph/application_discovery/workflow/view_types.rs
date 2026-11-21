//! Pure workflow history view wrapper structs.
use std::sync::Arc;
use worth_relational::facade::identity::EntityId;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowRecordedActor(pub EntityId);

impl WorthQueryWorkflowRecordedActor {
    pub fn observed(entity: EntityId) -> Self {
        Self(entity)
    }

    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

#[derive(Clone)]
pub struct WorthQueryWorkflowReadBasis(pub Arc<WorthQueryApplicationReadObservation>);

impl WorthQueryWorkflowReadBasis {
    pub fn from_observation(observation: Arc<WorthQueryApplicationReadObservation>) -> Self {
        Self(observation)
    }

    pub fn observation(&self) -> &WorthQueryApplicationReadObservation {
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
