//! Durable fork capacity names its exact source and destination before execution.
use crate::branch::owner_services::{SignalBranchForkReservation, SignalDurableBranchHandle};
use crate::durability::SignalDurableFrameDenial;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableSignalBranchIdentity, DurableSignalCellIncarnation,
};

#[must_use = "dropping this reservation releases the unperformed fork"]
pub struct SignalDurableForkReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) reservation: SignalBranchForkReservation<D, I, T>,
    pub(crate) source: SignalDurableBranchHandle,
    pub(crate) cell: DurableSignalCellIncarnation,
}
impl<D, I, T> SignalDurableForkReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn source(&self) -> &SignalDurableBranchHandle {
        &self.source
    }
    pub fn target(&self) -> DurableSignalBranchIdentity {
        self.cell.branch()
    }
    pub fn cell(&self) -> DurableSignalCellIncarnation {
        self.cell
    }
}

pub struct SignalCompositeForkBinding<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) reservation: SignalDurableForkReservation<D, I, T>,
    pub(crate) association: CompositeAttemptAssociation,
    pub(crate) commitment: [u8; 32],
}
impl<D, I, T> SignalCompositeForkBinding<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn commitment(&self) -> [u8; 32] {
        self.commitment
    }
    pub fn target(&self) -> DurableSignalBranchIdentity {
        self.reservation.target()
    }
    pub fn cell(&self) -> DurableSignalCellIncarnation {
        self.reservation.cell()
    }
    pub fn source(&self) -> &SignalDurableBranchHandle {
        self.reservation.source()
    }
    pub(crate) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        reservation: SignalDurableForkReservation<D, I, T>,
        association: CompositeAttemptAssociation,
    ) -> Result<Self, SignalDurableFrameDenial> {
        let commitment = crate::durability::fork_commitment(
            execution,
            &reservation.source,
            reservation.cell,
            &reservation.reservation.branch().name,
            &association,
        )?;
        Ok(Self {
            reservation,
            association,
            commitment,
        })
    }
}
