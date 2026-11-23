//! Checkpoint projection admission borrows an exact root already accepted by its source.
use super::RelationalBranchRoot;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::history::data::BranchId;
use crate::visibility::materialization::read_records::projection::RequestOwnedCheckpointBasis;
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalBranchRoot {
    pub(crate) fn admit_checkpoint_projection(
        self: &Arc<Self>,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        request: &ExecutionRequest<'_, '_>,
        branch: &BranchId,
    ) -> Result<RequestOwnedCheckpointBasis, Stop> {
        // The root is already admitted by its actual source owner. This operation
        // copies the observing branch name and shares that immutable root/schema.
        crate::durability::data::recovery_stop::account_work(request, branch.0.len() as u64 + 1)?;
        let memory = request
            .reserve_retained_memory(branch.0.len() as u64)
            .map_err(|cause| Stop::Request(cause.into()))?;
        // A readmitted root keeps its commit receipt even when its original
        // envelope is not resident; the receipt names the same version.
        let version = self
            .commit_receipt()
            .map(|receipt| receipt.version_id)
            .ok_or_else(|| {
                crate::durability::data::recovery_errors::DurabilityError::corrupt_checkpoint(
                    "index checkpoint root has no commit",
                )
            })?;
        Ok(RequestOwnedCheckpointBasis {
            basis: VisibilitySnapshotBasis::from_applied_root(
                branch.clone(),
                version,
                self.clone(),
            ),
            memory,
        })
    }
}
