//! One admitted operation through a weak binding to the actual state.
use std::sync::Weak;
use worth_execution::ExecutionRequest;

use super::{RelationalRuntime, RelationalRuntimeState};
use crate::durability::data::{
    CompleteRelationalNativeCheckpoint, RelationalCheckpointArtifactPolicy,
    RelationalNativeCheckpointCaptureDenial,
};

/// Retains no Runtime, Services, history or lifecycle binding while idle.
#[derive(Clone, Debug)]
pub struct CompleteRelationalCheckpointPreparation {
    state: Weak<RelationalRuntimeState>,
}

impl RelationalRuntime {
    pub fn complete_checkpoint_service(&self) -> CompleteRelationalCheckpointPreparation {
        CompleteRelationalCheckpointPreparation {
            state: self.state_binding(),
        }
    }
}

impl CompleteRelationalCheckpointPreparation {
    pub fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        artifact_policy: RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CompleteRelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial> {
        crate::execution::check_request(execution)
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
        let state = self
            .state
            .upgrade()
            .ok_or(RelationalNativeCheckpointCaptureDenial::OwnerUnavailable)?;
        let operation = state
            .owner_lifecycle
            .binding()
            .admit()
            .ok_or(RelationalNativeCheckpointCaptureDenial::OwnerUnavailable)?;
        let admitted = RelationalRuntime::admitted(state, operation);
        // The complete result takes all real holds before this operation ends.
        admitted
            .durability_authority()
            .complete_native_checkpoint_with_policy(execution, artifact_policy)
    }
}
