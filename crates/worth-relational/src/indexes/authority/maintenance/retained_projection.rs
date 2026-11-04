//! Explicit reconstruction can project an actually held recovered basis.
use super::*;
use crate::history::retention::RelationalRetentionOwnerRelationship;
use crate::identity::data::VersionId;
use worth_execution::ExecutionRequest;

impl IndexAuthority<'_> {
    pub(super) fn project_registered_retained_commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        version: VersionId,
        work: &mut MaintenanceWork,
    ) -> Result<Option<VisibilityProjectionView>, DerivedIndexMaintenanceDenialKind> {
        let Some(cell) = self.runtime.history.branch_cell(&request.branch_id) else {
            return Ok(None);
        };
        // Preserve this door's historical scope. Live-head work uses its exact basis.
        if cell.root().and_then(|root| root.commit_id()) == Some(request.source_commit_id) {
            return Ok(None);
        }
        let entries = cell
            .retained_bases_for_request(execution, |count| work.charge(count))
            .map_err(|cause| match cause {
                crate::branch::RelationalRetainedBasisSnapshotStop::Payment(cause) => cause,
                crate::branch::RelationalRetainedBasisSnapshotStop::Buffer(cause) => {
                    basis::buffer_stop(cause)
                }
                crate::branch::RelationalRetainedBasisSnapshotStop::Changed => {
                    DerivedIndexMaintenanceDenialKind::SnapshotUnavailable
                }
            })?;
        let owner = self.runtime.history.retention_binding();
        for entry in entries.iter() {
            work.prepare(3, 0)?;
            let Some(inner) = entry.upgrade() else {
                continue;
            };
            let actual = AdmittedRelationalBranchBasis { inner };
            let observation = actual.observation();
            if observation.commit_receipt().is_none_or(|receipt| {
                receipt.commit_id != request.source_commit_id || receipt.version_id != version
            }) {
                continue;
            }
            if actual.inner.retention.owner_relationship(&owner)
                != RelationalRetentionOwnerRelationship::SameOwner
            {
                continue;
            }
            let branch = observation.identity().branch_id();
            work.prepare(branch.0.len().min(request.branch_id.0.len()) as u64 + 1, 0)?;
            if branch != &request.branch_id {
                continue;
            }
            // VisibilitySnapshotBasis owns a copy of this exact selected name.
            work.prepare(branch.0.len() as u64 + 1, branch.0.len() as u64)?;
            return self
                .runtime
                .read_truth()
                .project_observation(&observation)
                .map(Some)
                .map_err(DerivedIndexMaintenanceDenialKind::Basis);
        }
        Ok(None)
    }
}
