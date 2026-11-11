//! Admit the original task's framework storage before running its callback.
use super::*;
use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, RetainedStorageCharge as Charge,
};
use crate::logic::transaction::{BranchState, TransactionResult};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) fn reserve<D, I, T>(
    execution: &ExecutionRequest<'_, '_>,
    branches: ReservedSignalBranches,
) -> Result<Option<ExecutionMemoryReservation>, SignalDurableAdvanceDenial>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    let charge = arc_allocation_charge::<Mutex<TaskSlot<D, I, T>>>()
        .and_then(|charge| {
            charge.checked_add(btree_structure_charge::<
                SignalBranchId,
                SignalBranchAdvanceIdentity,
            >(branches.count())?)
        })
        .and_then(|charge| {
            charge.checked_add(btree_structure_charge::<
                SignalBranchAdvanceIdentity,
                Arc<Mutex<TaskSlot<D, I, T>>>,
            >(1)?)
        })
        .and_then(|charge| charge.checked_add(Charge::capacity::<DefinitionPending<D, I, T>>(1)?))
        .and_then(|charge| charge.checked_add(Charge::capacity::<BranchPending<D, I, T>>(1)?))
        .and_then(|charge| {
            charge.checked_add(Charge::capacity::<SignalDurableAdvanceCompletion>(1)?)
        })
        .and_then(|charge| {
            charge.checked_add(match branches {
                ReservedSignalBranches::Advanced(_) => BranchState::<D, I, T>::durable_successor_framework_charge()?
                    .checked_add(Charge::capacity::<prepared::SignalPreparedDurableAdvance<D, I, T>>(1)?)?
                    .checked_add(Charge::capacity::<TransactionResult>(1)?)?,
                ReservedSignalBranches::Forked { .. } =>
                    crate::branch::owner_services::owner::fork_reservation::SignalPreparedOwnedFork::<D, I, T>::framework_charge()?,
            })
        })
        .map_err(|_| {
            SignalDurableAdvanceDenial::Readmission(
                crate::durability::SignalOwnerReadmissionDenial::MetadataCapacity,
            )
        })?;
    execution
        .reserve_retained_memory(charge.bytes())
        .map_err(SignalDurableAdvanceDenial::Request)
}
