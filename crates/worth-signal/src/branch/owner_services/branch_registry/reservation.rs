use std::sync::Arc;

use crate::state::SignalBranchId;

use super::super::cell_incarnation::SignalBranchCellIncarnation;
use super::super::lifecycle_state::SignalOwnerOperationAdmission;
use super::super::SignalBranchExecutionCell;
use super::{
    remove_name_for_branch, SignalBranchCellConstruction, SignalBranchRegistry,
    SignalBranchRegistryDenial, SignalBranchRegistryEntry,
};

#[must_use = "a branch reservation must be installed or dropped"]
#[derive(Debug)]
pub(crate) struct SignalBranchOwnedReservation<S> {
    registry: Option<Arc<SignalBranchRegistry<S>>>,
    branch_id: SignalBranchId,
    name: String,
    armed: bool,
}

impl<S> SignalBranchOwnedReservation<S> {
    pub(super) fn new(
        registry: Arc<SignalBranchRegistry<S>>,
        branch_id: SignalBranchId,
        name: String,
    ) -> Self {
        Self {
            registry: Some(registry),
            branch_id,
            name,
            armed: true,
        }
    }

    pub(crate) fn into_borrowed<'a>(
        mut self,
        registry: &'a SignalBranchRegistry<S>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
    ) -> SignalBranchReservation<'a, S> {
        let owned_registry = self
            .registry
            .take()
            .expect("an owned Signal branch reservation has one registry");
        assert!(
            std::ptr::eq(Arc::as_ptr(&owned_registry), registry),
            "an owned Signal branch reservation must return to its issuing registry"
        );
        drop(owned_registry);
        self.armed = false;
        SignalBranchReservation {
            registry,
            admission,
            branch_id: self.branch_id,
            prepared_cell_incarnation: None,
            reserved_name: Some(std::mem::take(&mut self.name)),
            consumed: false,
        }
    }
}

impl<S> Drop for SignalBranchOwnedReservation<S> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let Some(registry) = &self.registry else {
            return;
        };
        let mut state = registry.lock_state();
        if matches!(
            state.entries.get(&self.branch_id),
            Some(SignalBranchRegistryEntry::Reserved)
        ) {
            state.entries.remove(&self.branch_id);
            state.reservation_count = state
                .reservation_count
                .checked_sub(1)
                .expect("dropping a Signal branch reservation must release capacity");
            remove_name_for_branch(&mut state, self.branch_id);
        }
        self.armed = false;
    }
}

#[derive(Debug)]
pub(crate) struct SignalBranchReservation<'a, S> {
    pub(super) registry: &'a SignalBranchRegistry<S>,
    pub(super) admission: &'a SignalOwnerOperationAdmission<'a>,
    pub(super) branch_id: SignalBranchId,
    pub(super) prepared_cell_incarnation: Option<SignalBranchCellIncarnation>,
    pub(super) reserved_name: Option<String>,
    pub(super) consumed: bool,
}

impl<'a, S> SignalBranchReservation<'a, S> {
    #[cfg(test)]
    pub(super) fn unnamed(
        registry: &'a SignalBranchRegistry<S>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        branch_id: SignalBranchId,
    ) -> Self {
        Self {
            registry,
            admission,
            branch_id,
            prepared_cell_incarnation: None,
            reserved_name: None,
            consumed: false,
        }
    }

    pub(super) fn named(
        registry: &'a SignalBranchRegistry<S>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        branch_id: SignalBranchId,
        name: String,
    ) -> Self {
        Self {
            registry,
            admission,
            branch_id,
            prepared_cell_incarnation: None,
            reserved_name: Some(name),
            consumed: false,
        }
    }

    pub(crate) fn admission(&self) -> &'a SignalOwnerOperationAdmission<'a> {
        self.admission
    }

    pub(crate) fn install(
        self,
        state: S,
    ) -> Result<Arc<SignalBranchExecutionCell<S>>, SignalBranchRegistryDenial> {
        self.install_cell(state, false)
    }

    pub(crate) fn install_fork_destination(
        self,
        state: S,
    ) -> Result<Arc<SignalBranchExecutionCell<S>>, SignalBranchRegistryDenial> {
        self.install_cell(state, true)
    }

    pub(crate) fn prepare_fork_destination_cell(
        &mut self,
        state: S,
    ) -> Result<super::SignalPreparedBranchCell<S>, SignalBranchRegistryDenial> {
        self.prepare_cell_state(state, true)
    }

    pub(crate) fn matches_prepared_fork_destination(
        &self,
        prepared: &super::SignalPreparedBranchCell<S>,
    ) -> bool {
        prepared.is_fork_destination
            && self.prepared_cell_incarnation == Some(prepared.cell.incarnation())
    }

    pub(crate) fn bind_prepared_fork_destination(
        self,
        prepared: super::SignalPreparedBranchCell<S>,
    ) -> super::SignalPreparedBranchInstallation<'a, S> {
        assert!(
            self.matches_prepared_fork_destination(&prepared),
            "prepared fork destination must match its exact reservation"
        );
        super::SignalPreparedBranchInstallation {
            reservation: self,
            cell: prepared.cell,
            is_fork_destination: true,
        }
    }

    fn install_cell(
        self,
        state: S,
        is_fork_destination: bool,
    ) -> Result<Arc<SignalBranchExecutionCell<S>>, SignalBranchRegistryDenial> {
        self.prepare_cell(state, is_fork_destination)?.install()
    }

    fn prepare_cell(
        mut self,
        state: S,
        is_fork_destination: bool,
    ) -> Result<super::SignalPreparedBranchInstallation<'a, S>, SignalBranchRegistryDenial> {
        let prepared = self.prepare_cell_state(state, is_fork_destination)?;
        Ok(super::SignalPreparedBranchInstallation {
            reservation: self,
            cell: prepared.cell,
            is_fork_destination: prepared.is_fork_destination,
        })
    }

    fn prepare_cell_state(
        &mut self,
        state: S,
        is_fork_destination: bool,
    ) -> Result<super::SignalPreparedBranchCell<S>, SignalBranchRegistryDenial> {
        self.registry.validate_admission(self.admission)?;
        let cell = Arc::new(SignalBranchExecutionCell::new(
            SignalBranchCellConstruction(()),
            state,
            self.registry.owner_runtime_instance_id,
            self.registry.owner_lifecycle_identity,
            self.branch_id,
            Arc::clone(&self.registry.counters),
        ));
        if is_fork_destination {
            self.registry.counters.record_fork_destination_preparation();
        }
        self.prepared_cell_incarnation = Some(cell.incarnation());
        Ok(super::SignalPreparedBranchCell {
            cell,
            is_fork_destination,
        })
    }
}

impl<S> Drop for SignalBranchReservation<'_, S> {
    fn drop(&mut self) {
        if !self.consumed {
            debug_assert!(
                self.admission.permits_owner_lock_acquisition(),
                "branch reservation cleanup must run after target-cell release"
            );
            let mut state = self.registry.lock_state();
            if matches!(
                state.entries.get(&self.branch_id),
                Some(SignalBranchRegistryEntry::Reserved)
            ) {
                state.entries.remove(&self.branch_id);
                state.reservation_count = state
                    .reservation_count
                    .checked_sub(1)
                    .expect("dropping a Signal branch reservation must release capacity");
                remove_name_for_branch(&mut state, self.branch_id);
            }
        }
    }
}
