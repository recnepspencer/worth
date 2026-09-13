use crate::data::error::SignalError;
use crate::logic::transaction::runtime::state::runtime_state::{
    BranchLifecycleTransfer, RestoreTransferPacket, SignalRuntime,
};
use crate::state::{
    SignalBranchId, SignalSnapshotId, SignalSnapshotV1, SnapshotRestoreIntent, SnapshotRestorePlan,
};

use super::super::branches::{BranchState, SnapshotBranchState};

#[cfg(test)]
#[path = "restore_selection/tests.rs"]
mod tests;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn plan_snapshot_restore_for_target(
        &self,
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
    ) -> Result<SnapshotRestorePlan, SignalError> {
        if self.graph.current_branch().id == snapshot.meta.branch_id {
            return self.graph.plan_snapshot_restore(snapshot, intent);
        }
        if self
            .branches
            .branch_handle(snapshot.meta.branch_id)
            .is_none()
        {
            return self.graph.plan_snapshot_restore(snapshot, intent);
        }
        let target_state = self.snapshot_restore_target_state(snapshot.meta.branch_id)?;
        self.ensure_branch_state_managed_queue_transfer_allowed(target_state)?;
        target_state.graph().plan_snapshot_restore(snapshot, intent)
    }

    pub(super) fn load_snapshot_branch_state(
        &self,
        restored_branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    ) -> Result<Option<SnapshotBranchState<D, I, T>>, SignalError> {
        if let Some(snapshot_state) = self
            .branches
            .snapshot_state(restored_branch_id, snapshot_id)
        {
            return Ok(Some(snapshot_state.clone()));
        }
        if self.graph.current_branch().id == restored_branch_id {
            return Ok(None);
        }
        if self.branches.branch_handle(restored_branch_id).is_none() {
            return Ok(None);
        }
        let target_state = self.snapshot_restore_target_state(restored_branch_id)?;
        self.ensure_branch_state_managed_queue_transfer_allowed(target_state)?;
        Ok(Some(SnapshotBranchState::from_branch_state(target_state)))
    }

    pub(super) fn snapshot_restore_target_context(
        &self,
        restored_branch_id: SignalBranchId,
    ) -> Result<
        (
            crate::diagnostics::state::DiagnosticsState,
            crate::runtime_policy::InstalledSignalRuntimePolicy,
        ),
        SignalError,
    > {
        if self.graph.current_branch().id == restored_branch_id {
            return Ok((
                self.graph.diagnostics_state().clone(),
                self.graph.installed_runtime_policy(),
            ));
        }
        let target_state = self.snapshot_restore_target_state(restored_branch_id)?;
        self.ensure_branch_state_managed_queue_transfer_allowed(target_state)?;
        Ok((
            target_state.graph().diagnostics_state().clone(),
            target_state.graph().installed_runtime_policy(),
        ))
    }

    fn snapshot_restore_target_state(
        &self,
        restored_branch_id: SignalBranchId,
    ) -> Result<&BranchState<D, I, T>, SignalError> {
        let branch = self
            .branches
            .branch_handle(restored_branch_id)
            .ok_or_else(|| {
                SignalError::unknown_branch(Some(restored_branch_id), "snapshot-branch")
            })?;
        self.branches
            .branch_state(restored_branch_id)
            .ok_or_else(|| SignalError::unknown_branch(Some(restored_branch_id), branch.name))
    }

    pub(super) fn install_snapshot_restore_selection(
        &mut self,
        restored_branch_id: SignalBranchId,
        restored_state: BranchState<D, I, T>,
    ) -> Result<(), SignalError> {
        let ready = self.prepare_branch_lifecycle_transfer(BranchLifecycleTransfer::Restore(
            RestoreTransferPacket::new(restored_branch_id, restored_state),
        ))?;
        let outgoing_branch_id = self.graph.current_branch().id;
        let displaced_target = if outgoing_branch_id != restored_branch_id {
            let target_branch = self
                .branches
                .branch_handle(restored_branch_id)
                .expect("restore target context validated the live branch");
            let target_state = self
                .branches
                .branch_state(restored_branch_id)
                .ok_or_else(|| {
                    SignalError::unknown_branch(Some(restored_branch_id), target_branch.name)
                })?;
            self.ensure_branch_state_managed_queue_transfer_allowed(target_state)?;
            let outgoing_state = self.take_heavy_active_branch_state()?;
            self.branches.store_branch_state(outgoing_state);
            Some(
                self.branches
                    .take_stored_branch_transfer(restored_branch_id)
                    .expect("validated restore target remains stored until selection"),
            )
        } else {
            None
        };

        self.commit_branch_lifecycle_transfer(ready);
        drop(displaced_target);
        Ok(())
    }
}
