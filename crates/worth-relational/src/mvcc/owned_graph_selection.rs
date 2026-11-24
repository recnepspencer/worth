//! Retained graph selection enters the original Native proposal lifecycle.
use crate::mvcc::BranchBoundRelationalTransaction;

pub(crate) mod prepared;
pub(crate) use prepared::PreparedOwnedGraphSelection;

#[derive(Debug)]
pub struct RelationalOwnedGraphSelection {
    pub(crate) transaction: BranchBoundRelationalTransaction,
    pub(crate) retained: PreparedOwnedGraphSelection,
}
/// Admission and preparation failures preserve the original Native cause.
#[derive(Debug)]
pub enum RelationalOwnedGraphSelectionPreparationError {
    OwnerUnavailable,
    OwnershipNotInstalled {
        model: crate::identity::data::EntityId,
    },
    DurabilityUnsupported {
        model: crate::identity::data::EntityId,
    },
    TargetNotPristine,
    BasisChanged {
        model: crate::identity::data::EntityId,
    },
    Source(super::RelationalRetainedEntitySelectionDenial),
    Request(worth_execution::WorkCeilingDenial),
    Preparation(crate::transactions::data::TransactionCommitError),
}
