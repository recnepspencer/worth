//! The original root is the only authority to prepare, resume or commit a stop.
use super::root::SignalOwnerRootState;
use super::{SignalOwner, SignalOwnerRoot};
use crate::branch::SignalOwnerStopDenial as Denial;
use std::sync::atomic::Ordering;
use worth_execution::ExecutionRequest;
impl<D, I, T> SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn try_prepare_initial_empty_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.check_live().map_err(Denial::Stopped)?;
        self.sealed_stop_owner()?
            .try_prepare_initial_empty_stop(request)
    }
    pub(crate) fn try_resume_initial_empty_stop(&self) -> Result<(), Denial> {
        self.sealed_stop_owner()?
            .lifecycle
            .try_resume_initial_empty_stop()
    }
    pub(crate) fn try_commit_initial_empty_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.sealed_stop_owner()?.try_commit_owner_stop(request)
    }
    fn sealed_stop_owner(&self) -> Result<&SignalOwner<D, I, T>, Denial> {
        match &self.state {
            SignalOwnerRootState::Sealed(owner) => Ok(owner),
            SignalOwnerRootState::Unsealed { .. } => Err(Denial::Unsupported),
            SignalOwnerRootState::Transferred => Err(Denial::OwnerUnavailable),
        }
    }
}
impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn try_prepare_authored_home_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<super::super::lifecycle_state::SignalPreparedOwnerStopGate<'_>, Denial> {
        request.check_live().map_err(Denial::Stopped)?;
        let _metadata = self.metadata.try_authored_home_stop_guard()?;
        let _temporal = self.conditional_temporal.try_initial_empty_stop_guard()?;
        let _retention = self.retention.try_initial_empty_stop_guard()?;
        let _conditional = self.conditional_retention.try_initial_empty_stop_guard()?;
        request.check_live().map_err(Denial::Stopped)?;
        self.lifecycle.try_authored_stop_gate()
    }
    pub(super) fn try_commit_owner_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.check_live().map_err(Denial::Stopped)?;
        let _phase = self.lifecycle.try_prepared_initial_empty_stop_guard()?;
        let mut retention = self.retention.try_authored_home_close_guard()?;
        let mut conditional = self.conditional_retention.try_authored_home_close_guard()?;
        request.check_live().map_err(Denial::Stopped)?;
        // Nothing after this point may refuse or call cleanup/user code.
        retention.close();
        conditional.close();
        Ok(())
    }
    fn try_prepare_initial_empty_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let phase = self.lifecycle.try_initial_empty_stop_admission()?;
        let witness = self.initial_empty_root.get().ok_or(Denial::Unsupported)?;
        if self.selected_branch_id != witness.branch
            || self.next_branch_id.load(Ordering::Acquire) != 1
        {
            return Err(Denial::Unsupported);
        }
        let (cell, _registry) = self.registry.try_initial_empty_stop_cell(witness.branch)?;
        let _cell = cell.try_initial_empty_stop_guard(witness.incarnation, &witness.observation)?;
        let _metadata = self.metadata.try_initial_empty_stop_guard()?;
        let _retention = self.retention.try_initial_empty_stop_guard()?;
        let _conditional = self.conditional_retention.try_initial_empty_stop_guard()?;
        let _temporal = self.conditional_temporal.try_initial_empty_stop_guard()?;
        request.check_live().map_err(Denial::Stopped)?;
        phase.begin()
    }
}

#[cfg(test)]
mod tests;
