//! Bind durable dependency capture to the evidence's exact snapshot.
use super::{
    durable_dependencies::capture::capture, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;

impl RelationalRuntime {
    /// Capture exact source revisions from the evidence's own immutable root.
    /// Cost is O(tracked reads); no graph reconstruction is permitted here.
    pub fn capture_authorization_durable_dependencies(
        &self,
        evidence: &RelationalAuthorizationObservationEvidence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationDurableDependencies, Denial> {
        let view = self.project_authorization_dependency_snapshot(evidence.snapshot())?;
        capture(&view, execution, evidence)
    }

    pub(crate) fn project_authorization_dependency_snapshot(
        &self,
        snapshot: &SnapshotHandle,
    ) -> Result<VisibilityProjectionView, Denial> {
        if snapshot.runtime_instance_id != self.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        self.read_truth()
            .project_snapshot(snapshot)
            .ok_or(Denial::SnapshotUnavailable)
    }
}
