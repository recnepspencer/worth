//! Retained graph selection enters the original Native proposal lifecycle.
use crate::{
    branch::RelationalOwnedModelGraphSelection, history::data::CanonicalOwnedGraphSelection,
    mvcc::BranchBoundRelationalTransaction,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct RelationalOwnedGraphSelection {
    pub(crate) transaction: BranchBoundRelationalTransaction,
    pub(crate) retained: PreparedOwnedGraphSelection,
}
#[derive(Clone, Debug)]
pub(crate) struct PreparedOwnedGraphSelection {
    pub(crate) graph: Arc<RelationalOwnedModelGraphSelection>,
    pub(crate) meaning: Arc<CanonicalOwnedGraphSelection>,
    pub(crate) _funding: Arc<Option<worth_execution::ExecutionMemoryReservation>>,
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
