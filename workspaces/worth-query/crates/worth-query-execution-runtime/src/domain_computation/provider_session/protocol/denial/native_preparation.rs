//! Owned native diagnostics without recursively printing a potentially large commit log.
use worth_relational::facade::transactions::{CommitExecutionDenial, TransactionCommitError};

#[derive(Eq, PartialEq)]
pub(super) enum NativePreparationFailure {
    Commit(TransactionCommitError),
    Execution(CommitExecutionDenial),
}

impl std::fmt::Debug for NativePreparationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::Commit(error) => error,
            Self::Execution(denial) => return denial.fmt(formatter),
        };
        let category = match error {
            TransactionCommitError::Conflict { .. } => "Conflict",
            TransactionCommitError::Publication { .. } => "Publication",
            TransactionCommitError::Preparation { .. } => "Preparation",
            TransactionCommitError::Execution { .. } => "Execution",
            TransactionCommitError::Interrupted { .. } => "Interrupted",
            TransactionCommitError::PublicationDenied { .. } => "PublicationDenied",
            TransactionCommitError::PublicationDeferred { .. } => "PublicationDeferred",
            TransactionCommitError::PublicationFailed { .. } => "PublicationFailed",
            TransactionCommitError::DurabilityUnresolved { .. } => "DurabilityUnresolved",
        };
        let execution = match error {
            TransactionCommitError::Execution { denial, .. } => Some(denial),
            _ => None,
        };
        let context = error.context();
        formatter
            .debug_struct("NativePreparationFailure")
            .field("category", &category)
            .field("execution", &execution)
            .field("subsystem", &context.subsystem)
            .field("operation", &context.operation)
            .field("affected_record_count", &context.affected_records.len())
            .field(
                "commit_log_event_count",
                &error.commit_log().events().len(),
            )
            .finish()
    }
}
