//! Completion preparation retains an execution refusal at its original phase.
use super::WorthQueryInboundCompletionPreparationDenial as Denial;
use crate::domain_computation::primary_graph::provider::relational_execution_denial;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage as Stage;
use worth_relational::facade::transactions::{
    CommitExecutionDenialKind, TransactionCommitError as Error,
};

pub(in crate::domain_computation::primary_graph) fn validation_denial(error: &Error) -> Denial {
    commit_denial(
        error,
        Stage::InvariantExecution,
        Denial::ValidationUnavailable,
    )
}

pub(in crate::domain_computation::primary_graph) fn preparation_denial(error: &Error) -> Denial {
    commit_denial(error, Stage::ProviderCommit, Denial::PreparationUnavailable)
}

fn commit_denial(error: &Error, stage: Stage, ordinary: Denial) -> Denial {
    match error {
        Error::Execution { denial, .. } => {
            // A new public denial kind must force an explicit decision here.
            #[allow(clippy::infallible_destructuring_match)]
            let cause = match denial.kind {
                CommitExecutionDenialKind::Cause(cause) => cause,
            };
            match relational_execution_denial::relational_execution_kind(
                cause,
                denial.partition_identity,
            ) {
                Ok(kind) => Denial::ExecutionDenied { stage, kind },
                Err(kind) => Denial::ExecutionControlStopped { stage, kind },
            }
        }
        Error::Conflict { .. }
        | Error::Publication { .. }
        | Error::Preparation { .. }
        | Error::Interrupted { .. }
        | Error::PublicationDenied { .. }
        | Error::PublicationDeferred { .. }
        | Error::PublicationFailed { .. }
        | Error::PerformedButDurabilityDeferred { .. } => ordinary,
    }
}

#[cfg(test)]
mod tests;
