use std::sync::Arc;

use super::{
    map_metadata_hold_denial, mark_name_installed, remove_name_for_branch,
    SignalBranchExecutionCell, SignalBranchRegistryDenial, SignalBranchRegistryEntry,
    SignalBranchReservation,
};

pub(crate) struct SignalPreparedBranchCell<S> {
    pub(super) cell: Arc<SignalBranchExecutionCell<S>>,
    pub(super) is_fork_destination: bool,
}

pub(crate) struct SignalPreparedBranchInstallation<'a, S> {
    pub(super) reservation: SignalBranchReservation<'a, S>,
    pub(super) cell: Arc<SignalBranchExecutionCell<S>>,
    pub(super) is_fork_destination: bool,
}

pub(crate) struct SignalInstalledBranchCell<'a, S> {
    registry: &'a super::SignalBranchRegistry<S>,
    branch_id: crate::state::SignalBranchId,
    cell: Arc<SignalBranchExecutionCell<S>>,
    committed: bool,
}

impl<'a, S> SignalPreparedBranchInstallation<'a, S> {
    pub(crate) fn install(
        self,
    ) -> Result<Arc<SignalBranchExecutionCell<S>>, SignalBranchRegistryDenial> {
        let installed = self.install_recoverable()?;
        let cell = Arc::clone(installed.cell());
        installed.commit();
        Ok(cell)
    }

    pub(crate) fn install_recoverable(
        mut self,
    ) -> Result<SignalInstalledBranchCell<'a, S>, SignalBranchRegistryDenial> {
        self.reservation
            .registry
            .validate_admission(self.reservation.admission)?;
        let metadata_hold = self
            .reservation
            .admission
            .hold_owner_metadata()
            .map_err(map_metadata_hold_denial)?;
        let mut state = self.reservation.registry.lock_state();
        assert!(
            matches!(
                state.entries.get(&self.reservation.branch_id),
                Some(SignalBranchRegistryEntry::Reserved)
            ),
            "prepared Signal branch reservation must remain vacant"
        );
        if let Some(name) = self.reservation.reserved_name.as_deref() {
            mark_name_installed(&mut state, self.reservation.branch_id, name)?;
        }
        let entry = state
            .entries
            .get_mut(&self.reservation.branch_id)
            .expect("prepared Signal branch reservation must remain registered");
        *entry = SignalBranchRegistryEntry::Live(Arc::clone(&self.cell));
        state.reservation_count = state
            .reservation_count
            .checked_sub(1)
            .expect("prepared Signal branch installation must consume one reservation");
        state.live_count += 1;
        self.reservation.consumed = true;
        drop(state);
        drop(metadata_hold);
        if self.is_fork_destination {
            self.reservation
                .registry
                .counters
                .record_fork_destination_installation();
        }
        Ok(SignalInstalledBranchCell {
            registry: self.reservation.registry,
            branch_id: self.reservation.branch_id,
            cell: Arc::clone(&self.cell),
            committed: false,
        })
    }
}

impl<S> SignalInstalledBranchCell<'_, S> {
    pub(crate) fn cell(&self) -> &Arc<SignalBranchExecutionCell<S>> {
        &self.cell
    }

    pub(crate) fn commit(mut self) {
        self.committed = true;
    }
}

impl<S> Drop for SignalInstalledBranchCell<'_, S> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let mut state = self.registry.lock_state();
        let installed_matches = matches!(
            state.entries.get(&self.branch_id),
            Some(SignalBranchRegistryEntry::Live(cell)) if Arc::ptr_eq(cell, &self.cell)
        );
        if installed_matches {
            state.entries.remove(&self.branch_id);
            remove_name_for_branch(&mut state, self.branch_id);
            state.live_count = state
                .live_count
                .checked_sub(1)
                .expect("fork installation rollback releases exact live capacity");
        }
    }
}
