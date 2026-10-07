use crate::domain_computation::primary_graph::provider::relational_execution_denial;
use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{ConflictClass, InvariantViolationFields, TransactionCommitError},
};

use super::{
    WorthQueryGeneratedOutputInvariantAdmissionDenial,
    WorthQueryGeneratedOutputPublicationNoEffect, WorthQueryGeneratedOutputRestorationFailure,
};
use crate::domain_computation::primary_graph::WorthQuerySuspendedGeneratedOutput;

/// Why a restoration was rejected. Nothing was published.
#[derive(Debug)]
pub enum WorthQueryGeneratedOutputRestorationFailureCause {
    /// The suspended output belongs to another runtime or product.
    ForeignRuntime,
    /// The restoration named a different producer than the one that produced
    /// the output.
    WrongProducer,
    /// The producer's provider changed, or the producer is no longer installed.
    StaleProducerVersion,
    /// The runtime no longer records this output for its source.
    StaleOutputLineage,
    /// The product's program activation could not admit a publication.
    ProductActivationUnavailable,
    /// The prepared restoration's invariant evidence was not admitted.
    InvariantAdmission(WorthQueryGeneratedOutputInvariantAdmissionDenial),
    /// The execution owner refused restoration before publication.
    ExecutionDenied(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// The execution owner stopped restoration before publication.
    ExecutionControlStopped(crate::domain_computation::WorthQueryProviderSessionControlStopKind),
    /// The restoration could not be prepared.
    Preparation,
    /// A custom invariant rejected the prepared restoration; the fields name it
    /// and its version.
    PreparationInvariantRejected {
        identifier: String,
        major: u16,
        minor: u16,
    },
    /// The publication was refused or had no effect.
    PublicationNoEffect(WorthQueryGeneratedOutputPublicationNoEffect),
}

pub(super) fn preparation_failure_cause(
    error: &RelationalMaterializationError,
) -> WorthQueryGeneratedOutputRestorationFailureCause {
    use WorthQueryGeneratedOutputRestorationFailureCause as Cause;
    let error = match error {
        RelationalMaterializationError::Commit(error) => error,
        RelationalMaterializationError::OwnerUnavailable
        | RelationalMaterializationError::SourceCommitMismatch
        | RelationalMaterializationError::SourceIsNotCompleteCreatePublication
        | RelationalMaterializationError::SourceRecordUnavailable(_)
        | RelationalMaterializationError::BranchMismatch
        | RelationalMaterializationError::SuspensionCommitMismatch
        | RelationalMaterializationError::CandidateManifestMismatch
        | RelationalMaterializationError::PublicationResultMismatch
        | RelationalMaterializationError::TransactionAdmission(_)
        | RelationalMaterializationError::TransactionStaging(_) => return Cause::Preparation,
    };
    let error = match error {
        TransactionCommitError::Execution { denial, .. } => {
            // A new public denial kind must force an explicit decision here.
            #[allow(clippy::infallible_destructuring_match)]
            let cause = match denial.kind {
                worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) => {
                    cause
                }
            };
            return match relational_execution_denial::relational_execution_kind(
                cause,
                denial.partition_identity,
            ) {
                Ok(kind) => Cause::ExecutionDenied(kind),
                Err(kind) => Cause::ExecutionControlStopped(kind),
            };
        }
        TransactionCommitError::Conflict { error, .. } => error,
        TransactionCommitError::Publication { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::Interrupted { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. }
        | TransactionCommitError::PerformedButDurabilityDeferred { .. } => {
            return Cause::Preparation
        }
    };
    let ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::CustomInvariantViolation { identity },
        ..
    } = &error.class
    else {
        return WorthQueryGeneratedOutputRestorationFailureCause::Preparation;
    };
    WorthQueryGeneratedOutputRestorationFailureCause::PreparationInvariantRejected {
        identifier: identity.rule_id.as_str().to_owned(),
        major: identity.semantic_version.major,
        minor: identity.semantic_version.minor,
    }
}

pub(super) fn restoration_failure(
    suspended: WorthQuerySuspendedGeneratedOutput,
    cause: WorthQueryGeneratedOutputRestorationFailureCause,
) -> WorthQueryGeneratedOutputRestorationFailure {
    WorthQueryGeneratedOutputRestorationFailure::Rejected { suspended, cause }
}

#[cfg(test)]
mod tests;
