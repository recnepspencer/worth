use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{ConflictClass, InvariantViolationFields, TransactionCommitError},
};

use super::{
    WorthQueryGeneratedOutputInvariantAdmissionDenial, WorthQueryGeneratedOutputRestorationFailure,
};
use crate::domain_computation::primary_graph::WorthQuerySuspendedGeneratedOutput;

#[derive(Debug)]
pub enum WorthQueryGeneratedOutputRestorationFailureCause {
    ForeignRuntime,
    WrongProducer,
    StaleProducerVersion,
    StaleOutputLineage,
    ProductActivationUnavailable,
    InvariantAdmission(WorthQueryGeneratedOutputInvariantAdmissionDenial),
    Preparation,
    PreparationInvariantRejected {
        identifier: String,
        major: u16,
        minor: u16,
    },
    PublicationNoEffect,
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
