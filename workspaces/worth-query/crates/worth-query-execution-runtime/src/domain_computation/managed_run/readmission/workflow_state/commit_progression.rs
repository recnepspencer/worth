use super::*;

pub struct WorthQueryWorkflowCommittedAssociation {
    state: WorthQueryWorkflowReadmissionCommitState,
    resource: WorthQueryWorkflowRunRestoredPending,
    bridge: BridgeExecutionBasisReadmissionPending,
}

impl WorthQueryWorkflowCommittedAssociation {
    pub(super) fn owner_from_restored(
        restored: WorthQueryWorkflowRestoredAssociation,
        committed: WorthQueryArtifactProductionGenerationCommitted,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self {
            state: restored.state.commit_artifact_generation(committed),
            resource: restored.resource,
            bridge: restored.bridge,
        }
    }

    pub fn owner_commit(
        self,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> (
        crate::domain_computation::managed_run::WorthQueryRunningWorkflowRun,
        worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    ) {
        self.resource
            .commit_running(self.state, self.bridge, bridge_runtime, owner)
    }
}
