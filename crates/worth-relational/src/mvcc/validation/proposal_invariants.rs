use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

pub(super) fn stale_validated_proposal(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::StaleValidationBasis {
        detail: detail.to_owned(),
    }))
}
