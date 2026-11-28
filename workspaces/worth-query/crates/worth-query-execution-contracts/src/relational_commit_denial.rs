//! Routes Relational commit stops while borrowing the original owner evidence.

use crate::provider_session_denial_kind::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use crate::relational_execution_denial::relational_execution_kind;
use worth_relational::facade::{
    branch::RelationalBranchBasisDenial,
    mvcc::{
        RelationalInterruptionEvent, RelationalPublicationDeferred,
        RelationalPublicationFailureKind,
    },
    transactions::{
        CommitExecutionDenial, CommitExecutionDenialKind, ConflictClass, InvariantViolationFields,
        TransactionCommitError,
    },
};

pub enum RelationalCommitStop {
    Execution(CommitExecutionDenial),
    Interrupted(RelationalInterruptionEvent),
    Deferred(RelationalPublicationDeferred),
    PreEffect(Kind),
    RecoveryRequired,
}

pub fn classify_relational_commit(error: &TransactionCommitError) -> RelationalCommitStop {
    use RelationalCommitStop as Stop;
    use TransactionCommitError as Error;
    match error {
        Error::Execution { denial, .. } => Stop::Execution(*denial),
        Error::Interrupted { interruption, .. } => Stop::Interrupted(*interruption),
        Error::PublicationDeferred { deferred, .. } => Stop::Deferred(*deferred),
        Error::PublicationFailed { failure, .. } => Stop::PreEffect(match failure.kind() {
            RelationalPublicationFailureKind::SnapshotIdentityExhausted => {
                Kind::SnapshotIdentityExhausted
            }
            RelationalPublicationFailureKind::CandidateIdentityExhausted => {
                Kind::CandidateIdentityExhausted
            }
            RelationalPublicationFailureKind::RetentionIdentityExhausted => {
                Kind::RetentionIdentityExhausted
            }
            _ => Kind::ProviderRejected,
        }),
        Error::DurabilityUnresolved { .. } => Stop::RecoveryRequired,
        Error::Conflict { .. }
        | Error::Publication { .. }
        | Error::Preparation { .. }
        | Error::PublicationDenied { .. } => Stop::PreEffect(Kind::ProviderRejected),
    }
}

pub fn commit_execution_kind(denial: CommitExecutionDenial) -> Result<Kind, Control> {
    let CommitExecutionDenialKind::Cause(cause) = denial.kind;
    relational_execution_kind(cause, denial.partition_identity)
}

pub fn relational_invariant_fields(
    error: &TransactionCommitError,
) -> Option<&InvariantViolationFields> {
    match error {
        TransactionCommitError::Conflict { error, .. } => match &error.class {
            ConflictClass::InvariantViolation { fields, .. } => Some(fields),
            _ => None,
        },
        TransactionCommitError::Publication { error, .. } => error
            .invariant_violation()
            .map(|violation| &violation.fields),
        _ => None,
    }
}

pub fn relational_branch_basis_kind(denial: &RelationalBranchBasisDenial) -> Kind {
    match denial {
        RelationalBranchBasisDenial::RetentionCapacityExhausted => Kind::RetentionCapacityExhausted,
        RelationalBranchBasisDenial::RetentionIdentityExhausted => Kind::RetentionIdentityExhausted,
        RelationalBranchBasisDenial::SnapshotIdentityExhausted => Kind::SnapshotIdentityExhausted,
        _ => Kind::ProviderRejected,
    }
}

pub fn is_relation_law(fields: &InvariantViolationFields) -> bool {
    matches!(
        fields,
        InvariantViolationFields::RelationCardinalityEndpoint { .. }
            | InvariantViolationFields::RelationEndpointDeletionIntegrity { .. }
            | InvariantViolationFields::RelationEndpointKindSelfEdge { .. }
            | InvariantViolationFields::ConnectivityMinimum { .. }
            | InvariantViolationFields::PartitionIsolation { .. }
            | InvariantViolationFields::RelationSymmetry { .. }
            | InvariantViolationFields::RelationUniqueness { .. }
            | InvariantViolationFields::Acyclicity { .. }
    )
}
