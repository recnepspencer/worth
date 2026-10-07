//! Every re-dispatch refusal keeps its own recovery denial kind.

use std::collections::BTreeSet;

use super::WorthQueryExternalDispatchAttemptDenial as Attempt;
use super::WorthQueryExternalRedispatchDenial as Redispatch;
use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind as Kind,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial as Read;

const OWNER_READS: [Read; 13] = [
    Read::ForeignRuntime,
    Read::Missing,
    Read::AmbiguousCorrelation,
    Read::PendingPublication,
    Read::CommittedIndexUnavailable,
    Read::WrongRecordKind,
    Read::NotAuthoritative,
    Read::ExactCommitUnavailable,
    Read::ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: 3,
    },
    Read::SnapshotIdentityExhausted,
    Read::Malformed,
    Read::CommitMismatch,
    Read::RecordMismatch,
];

pub(super) const ATTEMPTS: [Attempt; 9] = [
    Attempt::ForeignRelationalRuntime,
    Attempt::ForeignProductWorld,
    Attempt::PublicationCommitMismatch,
    Attempt::InboundOperationSlotMissing,
    Attempt::AttemptIdentityExhausted,
    Attempt::OutstandingDispatchMissing,
    Attempt::OriginalPublicationPending,
    Attempt::OutstandingDispatchMismatch,
    Attempt::InFlightCapacityExhausted,
];

/// Every cause fresh effect authority reports for a live handle.
const FRESH_AUTHORITY: [Kind; 3] = [
    Kind::AlreadyTerminal,
    Kind::Expired,
    Kind::FreshAuthorityDenied,
];

/// The exhaustive match stops compiling when a refusal is added, so a new
/// refusal cannot skip this court.
fn every_refusal() -> Vec<Redispatch> {
    let listed = |denial: Redispatch| match denial {
        Redispatch::FreshAuthority(_)
        | Redispatch::AdmissionCancelled
        | Redispatch::AdmissionDeadlineExceeded
        | Redispatch::AdmissionAuthenticationExpired
        | Redispatch::ForeignAdmission
        | Redispatch::RecoveryNotAdmitted
        | Redispatch::BindingOutboxMissing
        | Redispatch::TransportNotInstalled
        | Redispatch::OwnerReadDenied(_)
        | Redispatch::AttemptAdmissionDenied(_)
        | Redispatch::AlreadyCompleted
        | Redispatch::CompletionPublicationPending
        | Redispatch::CompletionExecutionDenied { .. }
        | Redispatch::CompletionExecutionControlStopped { .. }
        | Redispatch::TerminalIndexUnavailable
        | Redispatch::CanonicalDerivationDenied
        | Redispatch::TimeObservationDenied => denial,
    };
    [
        Redispatch::AdmissionCancelled,
        Redispatch::AdmissionDeadlineExceeded,
        Redispatch::AdmissionAuthenticationExpired,
        Redispatch::ForeignAdmission,
        Redispatch::RecoveryNotAdmitted,
        Redispatch::BindingOutboxMissing,
        Redispatch::TransportNotInstalled,
        Redispatch::AlreadyCompleted,
        Redispatch::CompletionPublicationPending,
        Redispatch::CompletionExecutionDenied {
            stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit,
            kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionWorkerPanicked { partition_identity: Some(1) },
        },
        Redispatch::CompletionExecutionControlStopped {
            stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit,
            kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        },
        Redispatch::TerminalIndexUnavailable,
        Redispatch::CanonicalDerivationDenied,
        Redispatch::TimeObservationDenied,
    ]
    .into_iter()
    .chain(FRESH_AUTHORITY.map(Redispatch::FreshAuthority))
    .chain(OWNER_READS.map(Redispatch::OwnerReadDenied))
    .chain(ATTEMPTS.map(Redispatch::AttemptAdmissionDenied))
    .map(listed)
    .collect()
}

fn kind(denial: Redispatch) -> Kind {
    WorthQueryRecoveryHandleDenial::from(denial).kind()
}

#[test]
fn each_redispatch_refusal_names_its_own_recovery_kind() {
    let refusals = every_refusal();
    let kinds = refusals
        .iter()
        .map(|denial| format!("{:?}", kind(*denial)))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        kinds.len(),
        refusals.len(),
        "two refusals share one recovery kind: {kinds:?}"
    );
    assert!(
        refusals
            .iter()
            .all(|denial| kind(*denial) != Kind::CorrelationMismatch),
        "a re-dispatch refusal must not pose as a proof correlation mismatch"
    );
}

#[test]
fn settling_completion_refusals_keep_their_cause() {
    assert_eq!(
        kind(Redispatch::CompletionPublicationPending),
        Kind::CompletionPublicationPending
    );
    assert_eq!(
        kind(Redispatch::TerminalIndexUnavailable),
        Kind::TerminalIndexUnavailable
    );
    assert_eq!(kind(Redispatch::AlreadyCompleted), Kind::AlreadyCompleted);
    assert_eq!(
        kind(Redispatch::RecoveryNotAdmitted),
        Kind::RecoveryNotAdmitted
    );
    assert_eq!(
        kind(Redispatch::BindingOutboxMissing),
        Kind::DispatchOutboxMissing
    );
    assert_eq!(
        kind(Redispatch::TransportNotInstalled),
        Kind::TransportNotInstalled
    );
    for attempt in ATTEMPTS {
        assert_eq!(
            kind(Redispatch::AttemptAdmissionDenied(attempt)),
            Kind::AttemptAdmissionDenied(attempt),
            "the attempt cause {attempt:?} survives into the recovery kind"
        );
    }
    assert_eq!(
        kind(Redispatch::CanonicalDerivationDenied),
        Kind::CanonicalDerivationDenied
    );
    assert_eq!(
        kind(Redispatch::TimeObservationDenied),
        Kind::TimeObservationDenied
    );
    for read in OWNER_READS {
        assert_eq!(
            kind(Redispatch::OwnerReadDenied(read)),
            Kind::DispatchOwnerReadDenied(read),
            "the owner read cause {read:?} survives into the recovery kind"
        );
    }
}
#[test]
fn fresh_authority_and_current_admission_refusals_stay_distinct() {
    for fresh in FRESH_AUTHORITY {
        assert_eq!(
            kind(Redispatch::FreshAuthority(fresh)),
            fresh,
            "the fresh-authority cause {fresh:?} survives into the recovery kind"
        );
    }
    assert_eq!(
        kind(Redispatch::AdmissionCancelled),
        Kind::AdmissionCancelled
    );
    assert_eq!(
        kind(Redispatch::AdmissionDeadlineExceeded),
        Kind::AdmissionDeadlineExceeded
    );
    assert_eq!(
        kind(Redispatch::AdmissionAuthenticationExpired),
        Kind::AdmissionAuthenticationExpired
    );
    assert_eq!(kind(Redispatch::ForeignAdmission), Kind::ForeignRuntime);
}
