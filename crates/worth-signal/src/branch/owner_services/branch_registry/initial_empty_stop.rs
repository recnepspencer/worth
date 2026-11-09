//! The actual sole live cell remains in its original locked registry.
use super::{SignalBranchNameOccupancy, SignalBranchRegistry, SignalBranchRegistryEntry};
use crate::branch::SignalOwnerStopDenial as Denial;
use crate::state::SignalBranchId;
use std::sync::Arc;
impl<S> SignalBranchRegistry<S> {
    pub(in crate::branch::owner_services) fn try_initial_empty_stop_cell(
        &self,
        branch: SignalBranchId,
    ) -> Result<(Arc<super::SignalBranchExecutionCell<S>>, impl Drop + '_), Denial> {
        let state = self.state.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        if state.entries.len() != 1
            || state.live_count != 1
            || state.reservation_count != 0
            || state.names.len() != 1
            || state.names_by_branch.len() != 1
        {
            return Err(Denial::Unsupported);
        }
        let Some(SignalBranchRegistryEntry::Live(cell)) = state.entries.get(&branch) else {
            return Err(Denial::Unsupported);
        };
        let Some(name) = state.names_by_branch.get(&branch) else {
            return Err(Denial::Unsupported);
        };
        if !matches!(state.names.get(name), Some(SignalBranchNameOccupancy::Installed(id)) if *id == branch)
        {
            return Err(Denial::Unsupported);
        }
        // One temporary Arc increment; registry custody prevents last-drop here.
        let cell = Arc::clone(cell);
        Ok((cell, state))
    }
}
