//! Classification at the sole pre-publication native preparation port.
use super::transaction_commit_stop;
use crate::domain_computation::{
    WorthQueryProviderSessionCommitStop as Stop,
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Denial,
};
use worth_relational::facade::{
    errors::{ErrorContext, ErrorOperation, RelationalSubsystem},
    mvcc::TransactionCommitError,
    transactions::{
        CommitExecutionDenial, CommitExecutionDenialKind as Kind, CommitLog, CommitPhase,
    },
};

fn execution_error(kind: Kind) -> TransactionCommitError {
    let mut commit_log = CommitLog::new();
    commit_log.begin_phase(CommitPhase::DraftPreparation);
    TransactionCommitError::Execution {
        denial: CommitExecutionDenial {
            kind,
            partition_identity: Some(17),
        },
        context: ErrorContext::new(
            RelationalSubsystem::Transaction,
            ErrorOperation::ApplyMutation,
        ),
        commit_log,
    }
}

#[test]
fn every_execution_preparation_refusal_keeps_its_authoritative_posture() {
    for (kind, expected) in [
        (Kind::Cancelled, Control::Cancelled),
        (Kind::DeadlineElapsed, Control::TimedOut),
    ] {
        let Stop::ControlStopped(stopped) = transaction_commit_stop(execution_error(kind)) else {
            panic!("authoritative native control stop must remain control stopped");
        };
        assert_eq!(stopped.kind(), expected);
    }
    for kind in [
        Kind::Admission,
        Kind::ResourceExhausted,
        Kind::WorkExhausted,
        Kind::ResultCapacityExceeded,
        Kind::WorkerFailed,
    ] {
        let error = execution_error(kind);
        let Stop::PreEffectDenied(failure) = transaction_commit_stop(error.clone()) else {
            panic!("known preparation refusal must not become uncertain");
        };
        assert_eq!(failure.kind(), Denial::ProviderRejected);
        assert_eq!(failure.native_preparation_error(), Some(&error));
        let retained = failure.native_preparation_error().unwrap();
        assert_eq!(retained.context().operation, ErrorOperation::ApplyMutation);
        assert!(retained
            .commit_log()
            .has_phase_started(CommitPhase::DraftPreparation));
        let diagnostic = format!("{failure:?}");
        assert!(diagnostic.contains("WorkExhausted") == (kind == Kind::WorkExhausted));
        assert!(
            !diagnostic.contains("PhaseStarted"),
            "Debug must not recursively dump the native log"
        );
    }
}

#[test]
fn actual_native_deferred_reason_keeps_its_deferred_posture() {
    let error = TransactionCommitError::PublicationDeferred {
        deferred: worth_relational::facade::mvcc::RelationalPublicationDeferred::PatchPositionReservationContended,
        context: ErrorContext::new(RelationalSubsystem::Publication, ErrorOperation::Publish),
        commit_log: CommitLog::new(),
    };
    let Stop::Deferred(deferred) = transaction_commit_stop(error) else {
        panic!("native deferral is not a preparation denial");
    };
    assert_eq!(deferred.kind(), crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::PatchPositionReservationContended);
}
