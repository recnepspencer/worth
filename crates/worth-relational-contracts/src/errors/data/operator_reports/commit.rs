use super::*;
use crate::mvcc::publication::outcome::failure::{
    RelationalPublicationFailure, RelationalPublicationFailureKind,
};
use crate::mvcc::publication::outcome::refusals::{
    RelationalPublicationDeferred, RelationalPublicationDenial,
};
use crate::runtime::operation_control::interruption_event::RelationalOperationInterruption;
use crate::transactions::data::outcomes::{
    commit_conflict::CommitConflict,
    commit_preparation::{CommitPreparationError, CommitPreparationReason},
    conflict_class::ConflictClass,
    transaction_commit_error::{CommitExecutionDenialKind, TransactionCommitError},
};
use worth_execution::report::RetryHint;
use DenialClass::*;
use Disposition::*;

report!(TransactionCommitError, error => {
    use TransactionCommitError as Commit;
    let (class, disposition, code) = match error {
        Commit::Conflict { error, .. } => (error.denial_class(), error.disposition(), "relational.commit.conflict"),
        Commit::Publication { error, .. } => (error.denial_class(), error.disposition(), "relational.commit.publication"),
        Commit::Preparation { error, .. } => {
            let (class, disposition) = match error.reason() {
                CommitPreparationReason::SelectedBranchRoot(_) => (Conflict, Reauthorize),
                CommitPreparationReason::ProposalIdentityOrdinalExhausted
                | CommitPreparationReason::SymbolIdentityExhausted => (Exhausted, Final),
                CommitPreparationReason::StorageUnavailable(_) => (Unavailable, Escalate),
                CommitPreparationReason::ManagedIndexSource => (Unavailable, Final),
                CommitPreparationReason::SemanticResult(_) => (InvalidInput, CorrectInput),
            };
            (class, disposition, "relational.commit.preparation")
        }
        Commit::Execution { denial, .. } => {
            let CommitExecutionDenialKind::Cause(cause) = &denial.kind;
            (cause.denial_class(), cause.disposition(), "relational.commit.execution")
        }
        Commit::Interrupted { interruption, .. } => match interruption.interruption() {
            RelationalOperationInterruption::Cancelled => (Cancelled, Final, "relational.commit.cancelled"),
            RelationalOperationInterruption::TimedOut => (DeadlineElapsed, Final, "relational.commit.deadline"),
        },
        Commit::PublicationDenied { denial: RelationalPublicationDenial::CutoverDeadlineElapsed, .. } =>
            (DeadlineElapsed, Final, "relational.commit.cutover_deadline"),
        Commit::PublicationDenied { denial, .. } => {
            let (class, disposition) = super::publication_admission::admission_disposition(denial);
            (class, disposition, "relational.commit.publication_denied")
        }
        Commit::PublicationDeferred { deferred: RelationalPublicationDeferred::PatchPositionReservationContended, .. } =>
            (Unavailable, Retry(RetryHint::AfterBackoff), "relational.commit.reservation_contended"),
        Commit::PublicationDeferred { .. } => (Unavailable, Final, "relational.commit.publication_deferred"),
        Commit::PublicationFailed { failure, .. } => {
            let (class, disposition) = match failure.kind() {
                RelationalPublicationFailureKind::PreparedBasisDescriptor(cause)
                | RelationalPublicationFailureKind::NextBasisAdmission(cause)
                | RelationalPublicationFailureKind::BranchObservation(cause) => (cause.denial_class(), cause.disposition()),
                RelationalPublicationFailureKind::SnapshotIdentityExhausted
                | RelationalPublicationFailureKind::SnapshotCountOverflow
                | RelationalPublicationFailureKind::CandidateIdentityExhausted
                | RelationalPublicationFailureKind::PatchPositionCapacityExhausted
                | RelationalPublicationFailureKind::RetentionIdentityExhausted => (Exhausted, Final),
                RelationalPublicationFailureKind::SelectedRootUnavailable
                | RelationalPublicationFailureKind::RetentionOwner => (Unavailable, Escalate),
                RelationalPublicationFailureKind::OutputAdoptionRequired { .. } => (Authorization, Reauthorize),
                RelationalPublicationFailureKind::PreparedRootMismatch
                | RelationalPublicationFailureKind::PendingSettlementIdentityConflict => (Defect, Escalate),
            };
            (class, disposition, "relational.commit.publication_failed")
        }
        Commit::DurabilityUnresolved { .. } => (Indeterminate, Escalate, "relational.commit.durability_unresolved"),
    };
    (class, disposition, code, "The transaction did not complete.")
});
display_error!(TransactionCommitError, value => &value.detail(); value => match value {
    TransactionCommitError::Conflict { error, .. } => Some(error),
    TransactionCommitError::Publication { error, .. } => Some(error),
    TransactionCommitError::Preparation { error, .. } => Some(error),
    TransactionCommitError::Execution { denial, .. } => {
        let CommitExecutionDenialKind::Cause(cause) = &denial.kind;
        Some(cause)
    }
    TransactionCommitError::PublicationDenied { denial, .. } => Some(denial),
    TransactionCommitError::PublicationDeferred { deferred, .. } => Some(deferred),
    TransactionCommitError::PublicationFailed { failure, .. } => Some(failure),
    TransactionCommitError::DurabilityUnresolved { unresolved, .. } => Some(unresolved),
    TransactionCommitError::Interrupted { .. } => None,
});

report!(CommitConflict, error => {
    let class = &error.class;
    (class.denial_class(), class.disposition(), "relational.commit.conflict", class.diagnostic_summary())
});
display_error!(CommitConflict, error => &error.detail; error => Some(&error.class));

report!(ConflictClass, cause => {
    use ConflictClass as Cause;
    let (class, disposition, code) = match cause {
        Cause::ContentAdmission { denial } => (denial.denial_class(), denial.disposition(), "relational.conflict.content_admission"),
        Cause::StaleTarget { .. } | Cause::StaleValidationBasis { .. } =>
            (Conflict, Reauthorize, "relational.conflict.stale_basis"),
        Cause::DuplicateRelationIdentity { .. } | Cause::ConflictingIntent { .. } => (Conflict, CorrectInput, "relational.conflict.overlap"),
        Cause::ForeignRuntime { .. } | Cause::MaterializationAuthorityRequired =>
            (Authorization, Reauthorize, "relational.conflict.authority"),
        Cause::SymbolIdentityExhausted | Cause::ExecutionAllocationDenied { .. }
        | Cause::TransactionStagingCardinalityOverflow | Cause::TransactionInputDirectoryAllocationDenied { .. }
        | Cause::TransactionSavepointBudgetExceeded { .. } | Cause::TransactionSavepointIdentityExhausted
        | Cause::RecordAllocationDenied { .. } => (Exhausted, Final, "relational.conflict.capacity"),
        Cause::MutationStateInconsistency { .. } | Cause::EntityUpdateStateInconsistency { .. }
        | Cause::EntityFieldUpdateStateInconsistency { .. } | Cause::RelationEndpointUpdateStateInconsistency { .. }
        | Cause::RelationEndpointUpdateKindMismatch { .. } | Cause::EntityAuthoritativeAspectStateDenied { .. }
            => (Defect, Escalate, "relational.conflict.state"),
        Cause::InvariantViolation { .. } | Cause::RecordAspectPatchDenied { .. } | Cause::HistoricalReinterpretationViolation { .. }
        | Cause::AspectDeltaFailure { .. } => (Invariant, CorrectInput, "relational.conflict.invariant"),
        Cause::InvalidRelationEndpoint { .. } => (RelationLaw, CorrectInput, "relational.conflict.relation_endpoint"),
        Cause::KindSchemaMismatch { .. } | Cause::BulkImportDomainMismatch { .. } | Cause::InvalidSavepoint { .. }
        | Cause::InvalidMergeParent { .. } | Cause::MissingMergeBase { .. } | Cause::MaterializationModeMismatch
        | Cause::UndeclaredSchemaTransition { .. } | Cause::DescriptorSemanticsVersionUnsupported { .. }
        | Cause::InvalidSchemaTransitionSourceBasis { .. } | Cause::InvalidSchemaTransitionTargetBasis { .. }
        | Cause::MissingSchemaBasisForTransition { .. } | Cause::UnsupportedBridgeDescriptor { .. }
        | Cause::TypeContinuityDeniedSchemaTransition { .. } | Cause::StructuralContinuityDeniedSchemaTransition { .. }
        | Cause::DirectionalityMismatchUnderCanonicalReconciliation { .. } | Cause::InvalidSchemaTransitionShape { .. }
            => (InvalidInput, CorrectInput, "relational.conflict.invalid_input"),
    };
    (class, disposition, code, "Relational commit validation refused the proposed change.")
});
display_error!(ConflictClass, cause => &cause.detail(); cause => match cause {
    ConflictClass::ContentAdmission { denial } => Some(denial),
    ConflictClass::ExecutionAllocationDenied { denial } => Some(denial),
    ConflictClass::InvariantViolation { fields: crate::validation::data::results::InvariantViolationFields::CustomInvariantViolation { refusal, .. }, .. } => Some(refusal),
    _ => None,
});

display_error!(CommitPreparationError, error => &error.detail(); _error => None);
display_error!(RelationalPublicationDeferred, _error => "Publication was deferred."; _error => None);
display_error!(RelationalPublicationFailure, error => error.detail(); error => match error.kind() {
    RelationalPublicationFailureKind::PreparedBasisDescriptor(cause)
    | RelationalPublicationFailureKind::NextBasisAdmission(cause)
    | RelationalPublicationFailureKind::BranchObservation(cause) => Some(cause),
    _ => None,
});
display_error!(crate::durability::medium::unresolved::RelationalDurabilityUnresolved, error => error.detail(); _error => None);
