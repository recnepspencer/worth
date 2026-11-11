//! Captured fork custody survives publication without borrowing an operation admission.
use super::*;
use crate::branch::owner_services::branch_registry::SignalBranchOwnedReservation;
use crate::branch::owner_services::owner_metadata::SignalOwnerOwnedForkLineageReservation;
use crate::branch::owner_services::SignalOwner;

pub(in crate::branch::owner_services) struct SignalPreparedOwnedFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Arc<SignalOwner<D, I, T>>,
    handle: SignalBranchHandle,
    registry: SignalBranchOwnedReservation<SignalBranchCellState<D, I, T>>,
    lineage: SignalOwnerOwnedForkLineageReservation<D, I, T>,
    cell: SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
    observation: SignalBranchObservation,
}
impl<D, I, T> SignalPreparedOwnerFork<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn into_owned(
        self,
        owner: Arc<SignalOwner<D, I, T>>,
    ) -> SignalPreparedOwnedFork<D, I, T> {
        let (registry, cell) = self.installation.into_owned(Arc::clone(owner.registry()));
        let lineage = self.lineage.into_owned(Arc::clone(&owner));
        SignalPreparedOwnedFork {
            owner,
            handle: self.handle,
            registry,
            lineage,
            cell,
            observation: self.destination_observation,
        }
    }
}
impl<D, I, T> SignalPreparedOwnedFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn framework_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        use crate::data::retained_storage::{
            arc_allocation_charge, RetainedStorageCharge as Charge,
        };
        arc_allocation_charge::<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>()?
            .checked_add(Charge::capacity::<Self>(1)?)?
            .checked_add(BranchState::<D, I, T>::forked_framework_charge()?)
    }
    pub(in crate::branch::owner_services) fn branch(&self) -> &SignalBranchHandle {
        &self.handle
    }
    pub(in crate::branch::owner_services) fn observation(&self) -> &SignalBranchObservation {
        &self.observation
    }
    pub(in crate::branch::owner_services) fn state_mut(
        &mut self,
    ) -> &mut SignalBranchCellState<D, I, T> {
        self.cell.state_mut()
    }
    pub(in crate::branch::owner_services) fn install<'a>(
        self,
        owner: &'a Arc<SignalOwner<D, I, T>>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
    ) -> Result<SignalInstalledOwnerFork<'a, D, I, T>, SignalBranchForkOperationDenial> {
        assert!(
            Arc::ptr_eq(owner, &self.owner),
            "captured fork retains its original owner"
        );
        let reservation = self.registry.into_borrowed(owner.registry(), admission);
        let installation = reservation.bind_prepared_fork_destination(self.cell);
        let lineage = self.lineage.into_borrowed(&owner.metadata, admission);
        SignalPreparedOwnerFork {
            handle: self.handle,
            admission,
            installation,
            lineage,
            destination_observation: self.observation,
        }
        .install()
    }
}
