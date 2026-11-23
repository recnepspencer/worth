//! Bind durable dependency capture to the evidence's exact snapshot.
use super::{
    durable_dependencies::capture::capture, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Capture exact source revisions from the evidence's own immutable root.
    /// Cost is O(tracked reads); no graph reconstruction is permitted here.
    pub fn capture_authorization_durable_dependencies(
        &self,
        evidence: &RelationalAuthorizationObservationEvidence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationDurableDependencies, Denial> {
        if evidence.snapshot().runtime_instance_id != self.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        let view = self
            .read_truth()
            .project_snapshot(evidence.snapshot())
            .ok_or(Denial::SnapshotUnavailable)?;
        capture(&view, execution, evidence)
    }
}
