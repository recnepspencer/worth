//! Owned native diagnostics without recursively printing a potentially large commit log.
use worth_relational::facade::mvcc::TransactionCommitError;

#[derive(Eq, PartialEq)]
pub(super) struct NativePreparationFailure(pub(super) TransactionCommitError);

impl std::fmt::Debug for NativePreparationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let category = match &self.0 {
            TransactionCommitError::Conflict { .. } => "Conflict",
            TransactionCommitError::Publication { .. } => "Publication",
            TransactionCommitError::Preparation { .. } => "Preparation",
            TransactionCommitError::Execution { .. } => "Execution",
            TransactionCommitError::Interrupted { .. } => "Interrupted",
            TransactionCommitError::PublicationDenied { .. } => "PublicationDenied",
            TransactionCommitError::PublicationDeferred { .. } => "PublicationDeferred",
            TransactionCommitError::PublicationFailed { .. } => "PublicationFailed",
            TransactionCommitError::PerformedButDurabilityDeferred { .. } => {
                "PerformedButDurabilityDeferred"
            }
        };
        let execution = match &self.0 {
            TransactionCommitError::Execution { denial, .. } => Some(denial),
            _ => None,
        };
        let context = self.0.context();
        formatter
            .debug_struct("NativePreparationFailure")
            .field("category", &category)
            .field("execution", &execution)
            .field("subsystem", &context.subsystem)
            .field("operation", &context.operation)
            .field("affected_record_count", &context.affected_records.len())
            .field(
                "commit_log_event_count",
                &self.0.commit_log().events().len(),
            )
            .finish()
    }
}
