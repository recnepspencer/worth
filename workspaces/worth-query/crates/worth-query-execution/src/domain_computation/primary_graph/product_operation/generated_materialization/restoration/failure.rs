use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{ConflictClass, InvariantViolationFields, TransactionCommitError},
};

use super::{
    WorthQueryGeneratedOutputInvariantAdmissionDenial,
    WorthQueryGeneratedOutputPublicationNoEffect, WorthQueryRestoredGeneratedOutput,
    WorthQueryUnpublishedGeneratedOutputRestoration,
};
use crate::domain_computation::primary_graph::WorthQuerySuspendedGeneratedOutput;

/// Why a restoration was rejected. Nothing was published.
#[derive(Debug)]
pub enum WorthQueryGeneratedOutputRestorationFailureCause {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
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
    let RelationalMaterializationError::Commit(TransactionCommitError::Conflict { error, .. }) =
        error
    else {
        return WorthQueryGeneratedOutputRestorationFailureCause::Preparation;
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

/// Why restoring a generated output did not complete.
pub enum WorthQueryGeneratedOutputRestorationFailure {
    /// Publication completed, but the sealed owner refused lineage admission.
    Handle {
        denial: crate::facade::primary_graph::WorthQueryHandleDenial,
        restored: WorthQueryRestoredGeneratedOutput,
    },
    /// Nothing was published. The suspended output is handed back so it can be
    /// reconstructed again; the cause says why.
    Rejected {
        suspended: WorthQuerySuspendedGeneratedOutput,
        cause: WorthQueryGeneratedOutputRestorationFailureCause,
    },
    /// Some owners moved, but the product head did not. Continue the recovery
    /// this carries.
    ProductUnpublished(WorthQueryUnpublishedGeneratedOutputRestoration),
}

impl WorthQueryGeneratedOutputRestorationFailure {
    pub fn cause(&self) -> Option<&WorthQueryGeneratedOutputRestorationFailureCause> {
        match self {
            Self::Rejected { cause, .. } => Some(cause),
            Self::ProductUnpublished(_) | Self::Handle { .. } => None,
        }
    }

    pub fn into_suspended(self) -> Result<WorthQuerySuspendedGeneratedOutput, Self> {
        match self {
            Self::Rejected { suspended, .. } => Ok(suspended),
            unpublished @ (Self::ProductUnpublished(_) | Self::Handle { .. }) => Err(unpublished),
        }
    }
}
