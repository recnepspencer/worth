//! Invariant validation preserves the original Relational refusal and projects Query's posture.
use std::sync::Arc;

use super::{
    InvariantViolationFields, NativeDenial, WorthQueryInvariantExecutionDenialKind as Kind,
    WorthQueryInvariantExecutionFailure as Failure,
    WorthQueryInvariantExecutionFailurePosture as Posture,
};
use worth_query_execution_contracts::facade::{
    classify_relational_commit, commit_execution_kind, relational_invariant_fields,
    RelationalCommitStop as Stop, WorthQueryProviderSessionDenialKind as Session,
};
use worth_relational::facade::{
    mvcc::RelationalPublicationDeferred as Deferred,
    transactions::{CommitExecutionDenial, CommitPreparationReason, TransactionCommitError},
};

impl Failure {
    pub fn relational_validation(error: TransactionCommitError) -> Self {
        let native = NativeDenial::Validation(Arc::new(error));
        let NativeDenial::Validation(error) = &native else {
            unreachable!()
        };
        let (kind, posture) = if native.allocation().is_some() {
            native.classification()
        } else {
            validation_classification(error)
        };
        Self {
            kind,
            posture,
            detail: error.detail().into(),
            native: Some(native),
            affected_records: None,
        }
    }

    pub fn relational_execution(denial: CommitExecutionDenial) -> Self {
        let mut failure = Self::new(execution_kind(denial), format!("{denial:?}"));
        failure.native = Some(NativeDenial::Execution(denial));
        failure
    }

    pub fn relational_execution_denial(&self) -> Option<CommitExecutionDenial> {
        match self.native.as_ref()? {
            NativeDenial::Execution(denial) => Some(*denial),
            NativeDenial::Validation(error) => match classify_relational_commit(error) {
                Stop::Execution(denial) => Some(denial),
                _ => None,
            },
            _ => None,
        }
    }
}

fn validation_classification(error: &TransactionCommitError) -> (Kind, Posture) {
    let exhausted = match classify_relational_commit(error) {
        Stop::Execution(denial) => return (execution_kind(denial), Posture::Denied),
        Stop::Interrupted(event) => {
            return (
                Kind::RequestInterrupted(event.interruption()),
                Posture::Denied,
            )
        }
        Stop::Deferred(deferred) => deferred_kind(deferred),
        Stop::PreEffect(Session::SnapshotIdentityExhausted) => Kind::SnapshotIdentityExhausted,
        Stop::PreEffect(Session::CandidateIdentityExhausted) => Kind::CandidateIdentityExhausted,
        Stop::PreEffect(Session::RetentionIdentityExhausted) => Kind::RetentionIdentityExhausted,
        Stop::PreEffect(_) | Stop::RecoveryRequired => return classify_preparation(error),
    };
    (exhausted, Posture::Exhausted)
}

fn execution_kind(denial: CommitExecutionDenial) -> Kind {
    match commit_execution_kind(denial) {
        Ok(kind) => Kind::ExecutionDenied(kind),
        Err(kind) => Kind::ExecutionControlStopped(kind),
    }
}

fn classify_preparation(error: &TransactionCommitError) -> (Kind, Posture) {
    if matches!(error, TransactionCommitError::Preparation { error, .. }
        if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted)
    {
        return (Kind::ProposalIdentityExhausted, Posture::Exhausted);
    }
    let kind = match relational_invariant_fields(error) {
        Some(
            InvariantViolationFields::CustomInvariantViolation { .. }
            | InvariantViolationFields::CustomInvariantFailure { .. },
        ) => Kind::CustomInvariantDenied,
        _ => Kind::ProviderRejected,
    };
    (kind, Posture::Denied)
}

fn deferred_kind(deferred: Deferred) -> Kind {
    match deferred {
        Deferred::CompanionRegistrationPending
        | Deferred::CompanionRebindRequired
        | Deferred::CompanionPreflight(_) => Kind::RelationalDeferred(deferred),
        Deferred::RetentionBackpressure => Kind::RetentionCapacityExhausted,
        Deferred::PatchPositionReservationContended => Kind::PatchPositionReservationContended,
        Deferred::CandidateCapacityExhausted { maximum_candidates } => {
            Kind::CandidateCapacityExhausted { maximum_candidates }
        }
        Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
            Kind::PublishedSnapshotCapacityExhausted { maximum_handles }
        }
    }
}
