use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{ConflictClass, InvariantViolationFields, TransactionCommitError},
};

use crate::domain_computation::primary_graph::WorthQuerySuspendedGeneratedOutput;
use {
    super::WorthQueryGeneratedOutputPublicationNoEffect,
    super::WorthQueryGeneratedOutputRestorationFailure,
    crate::domain_computation::vocabulary::primary_graph::invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial,
};

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
    ExecutionDenied(crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind),
    /// The execution owner stopped restoration before publication.
    ExecutionControlStopped(crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind),
    /// The restoration could not be prepared.
    Preparation,
    /// A custom invariant rejected the prepared restoration; the fields name it
    /// and its version.
    PreparationInvariantRejected {
        refusal: worth_relational::facade::runtime::CustomInvariantRefusal,
        identifier: String,
        major: u16,
        minor: u16,
    },
    /// The publication was refused or had no effect.
    PublicationNoEffect(WorthQueryGeneratedOutputPublicationNoEffect),
}

pub fn preparation_failure_cause(
    error: &RelationalMaterializationError,
) -> WorthQueryGeneratedOutputRestorationFailureCause {
    use WorthQueryGeneratedOutputRestorationFailureCause as Cause;
    if let Some(execution) = super::super::preparation_execution::project(error) {
        return match execution {
            Ok(kind) => Cause::ExecutionDenied(kind),
            Err(kind) => Cause::ExecutionControlStopped(kind),
        };
    }
    let error = match error {
        RelationalMaterializationError::Commit(error) => error,
        RelationalMaterializationError::Read(_)
        | RelationalMaterializationError::OwnerUnavailable
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
        TransactionCommitError::Conflict { error, .. } => error,
        TransactionCommitError::Execution { .. }
        | TransactionCommitError::Publication { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::Interrupted { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. }
        // Restoration preparation stops before publication.
        | TransactionCommitError::DurabilityUnresolved { .. } => {
            return Cause::Preparation
        }
    };
    let ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::CustomInvariantViolation { identity, refusal },
        ..
    } = &error.class
    else {
        return WorthQueryGeneratedOutputRestorationFailureCause::Preparation;
    };
    WorthQueryGeneratedOutputRestorationFailureCause::PreparationInvariantRejected {
        refusal: refusal.clone(),
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

impl WorthQueryGeneratedOutputRestorationFailureCause {
    pub(super) fn advancement(
        cause: crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
    ) -> Self {
        match crate::domain_computation::provider_session::protocol::advancement_denial_cause(cause)
        {
            Ok(kind) => Self::ExecutionDenied(kind),
            Err(kind) => Self::ExecutionControlStopped(kind),
        }
    }
}
