//! The original synchronous callback result remains beside its isolated candidate.
use crate::branch::SignalBranchObservation;
use crate::durability::SignalDurableBranchDescriptor;
use crate::logic::transaction::{BranchState, TransactionResult};
use crate::state::SignalSnapshotId;

pub(in crate::branch::owner_services) struct SignalPreparedDurableAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) state: Box<BranchState<D, I, T>>,
    pub(in crate::branch::owner_services) predecessor: SignalDurableBranchDescriptor,
    pub(in crate::branch::owner_services) observation: SignalBranchObservation,
    pub(in crate::branch::owner_services) generation: u64,
    pub(in crate::branch::owner_services) restore_snapshot: Option<SignalSnapshotId>,
    pub(in crate::branch::owner_services) source_changed: bool,
    pub(in crate::branch::owner_services) transaction: Box<TransactionResult>,
}
