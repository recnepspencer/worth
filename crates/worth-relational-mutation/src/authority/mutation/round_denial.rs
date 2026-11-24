//! Provisional symbol discovery is consumed by the candidate round orchestrator.
#[derive(Debug)]
pub enum MutationApplyDenial {
    Commit(crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
    Symbols(crate::storage::runtime::state::subsystems::services::symbol_table::reservations::SymbolReservationDenial),
}

impl From<crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict>
    for MutationApplyDenial
{
    fn from(
        conflict: crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict,
    ) -> Self {
        Self::Commit(crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
            conflict,
        ))
    }
}

impl From<crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>
    for MutationApplyDenial
{
    fn from(
        error: crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    ) -> Self {
        Self::Commit(error)
    }
}

impl From<crate::storage::runtime::state::subsystems::services::symbol_table::reservations::SymbolReservationDenial>
    for MutationApplyDenial
{
    fn from(
        error: crate::storage::runtime::state::subsystems::services::symbol_table::reservations::SymbolReservationDenial,
    ) -> Self {
        Self::Symbols(error)
    }
}
