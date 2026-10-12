//! Current snapshot reads under caller custody.
use super::WorthQueryWorkspace;
use crate::memory_workspace::WorthQuerySnapshotIdentity;

impl WorthQueryWorkspace {
    pub fn snapshot_identity(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
    ) -> Result<
        WorthQuerySnapshotIdentity,
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    > {
        let execution =
            phase.checked_execution_request_for(&self.runtime.installed_product.world)?;
        Ok(self.snapshot_identity_with_request(&execution))
    }

    pub(crate) fn snapshot_identity_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQuerySnapshotIdentity {
        self.runtime
            .current_snapshot_identity_with_request(execution)
    }
}
