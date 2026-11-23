//! Provisional symbol discovery is consumed by the candidate round orchestrator.
#[derive(Debug)]
pub(crate) enum MutationApplyDenial {
    Commit(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
    Symbols(crate::runtime::state::subsystems::services::symbol_table::SymbolReservationDenial),
}

impl From<crate::transactions::data::outcomes::commit_conflict::CommitConflict>
    for MutationApplyDenial
{
    fn from(
        conflict: crate::transactions::data::outcomes::commit_conflict::CommitConflict,
    ) -> Self {
        Self::Commit(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
            conflict,
        ))
    }
}

impl From<crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>
    for MutationApplyDenial
{
    fn from(
        error: crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    ) -> Self {
        Self::Commit(error)
    }
}

impl From<crate::runtime::state::subsystems::services::symbol_table::SymbolReservationDenial>
    for MutationApplyDenial
{
    fn from(
        error: crate::runtime::state::subsystems::services::symbol_table::SymbolReservationDenial,
    ) -> Self {
        Self::Symbols(error)
    }
}
