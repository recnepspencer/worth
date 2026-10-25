//! Provisional symbol discovery is consumed by the candidate round orchestrator.
#[derive(Debug)]
pub(crate) enum MutationApplyDenial {
    Commit(crate::transactions::data::TransactionCommitError),
    Symbols(crate::runtime::SymbolReservationDenial),
}

impl From<crate::transactions::data::CommitConflict> for MutationApplyDenial {
    fn from(conflict: crate::transactions::data::CommitConflict) -> Self {
        Self::Commit(crate::transactions::data::TransactionCommitError::conflict(
            conflict,
        ))
    }
}

impl From<crate::transactions::data::TransactionCommitError> for MutationApplyDenial {
    fn from(error: crate::transactions::data::TransactionCommitError) -> Self {
        Self::Commit(error)
    }
}

impl From<crate::runtime::SymbolReservationDenial> for MutationApplyDenial {
    fn from(error: crate::runtime::SymbolReservationDenial) -> Self {
        Self::Symbols(error)
    }
}
