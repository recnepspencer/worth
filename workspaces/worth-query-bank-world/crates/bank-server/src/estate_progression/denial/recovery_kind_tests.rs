//! Bank keeps each Query recovery refusal as its own kind.

use std::collections::BTreeSet;

use worth_query_host::facade::primary_graph::{
    WorthQueryCommittedDispatchOutboxReadDenial as Read,
    WorthQueryExternalDispatchAttemptDenial as Attempt, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind as Query,
};

use super::{BankRecoveryDenial, BankRecoveryDenialKind as Bank};
use crate::BankCommittedDispatchOutboxReadDenial as BankRead;
use crate::BankExternalDispatchAttemptDenial as BankAttempt;

fn bank(kind: Query) -> Bank {
    BankRecoveryDenial::from_query(WorthQueryRecoveryHandleDenial::new(kind)).kind()
}

#[test]
fn redispatch_refusals_keep_distinct_bank_kinds() {
    let refusals = [
        (Query::AlreadyCompleted, Bank::AlreadyCompleted),
        (
            Query::CompletionPublicationPending,
            Bank::CompletionPublicationPending,
        ),
        (
            Query::TerminalIndexUnavailable,
            Bank::TerminalIndexUnavailable,
        ),
        (Query::DispatchOutboxMissing, Bank::DispatchOutboxMissing),
        (Query::TransportNotInstalled, Bank::TransportNotInstalled),
        (
            Query::DispatchOwnerReadDenied(Read::ExactCommitUnavailable),
            Bank::DispatchOwnerReadDenied(BankRead::ExactCommitUnavailable),
        ),
        (
            Query::AttemptAdmissionDenied(Attempt::InFlightCapacityExhausted),
            Bank::AttemptAdmissionDenied(BankAttempt::InFlightCapacityExhausted),
        ),
        (
            Query::AttemptAdmissionDenied(Attempt::OutstandingDispatchMismatch),
            Bank::AttemptAdmissionDenied(BankAttempt::OutstandingDispatchMismatch),
        ),
        (
            Query::CanonicalDerivationDenied,
            Bank::CanonicalDerivationDenied,
        ),
        (Query::TimeObservationDenied, Bank::TimeObservationDenied),
        (Query::RecoveryNotAdmitted, Bank::RecoveryNotAdmitted),
        (Query::FreshAuthorityDenied, Bank::FreshAuthorityDenied),
        (Query::AdmissionCancelled, Bank::AdmissionCancelled),
        (
            Query::AdmissionDeadlineExceeded,
            Bank::AdmissionDeadlineExceeded,
        ),
        (
            Query::AdmissionAuthenticationExpired,
            Bank::AdmissionAuthenticationExpired,
        ),
        (
            Query::DispatchOwnerReadDenied(Read::PendingPublication),
            Bank::DispatchOwnerReadDenied(BankRead::PendingPublication),
        ),
        (
            Query::DispatchOwnerReadDenied(Read::CommittedIndexUnavailable),
            Bank::DispatchOwnerReadDenied(BankRead::CommittedIndexUnavailable),
        ),
        (
            Query::DispatchOwnerReadDenied(Read::AmbiguousCorrelation),
            Bank::DispatchOwnerReadDenied(BankRead::AmbiguousCorrelation),
        ),
    ];
    for (query, expected) in refusals {
        assert_eq!(bank(query), expected, "{query:?} lost its cause in Bank");
    }
    let distinct = refusals
        .iter()
        .map(|(query, _)| format!("{:?}", bank(*query)))
        .collect::<BTreeSet<_>>();
    assert_eq!(distinct.len(), refusals.len());
}

#[test]
fn every_attempt_refusal_keeps_its_own_bank_cause() {
    let attempts = [
        (
            Attempt::ForeignRelationalRuntime,
            BankAttempt::ForeignRelationalRuntime,
        ),
        (
            Attempt::ForeignProductWorld,
            BankAttempt::ForeignProductWorld,
        ),
        (
            Attempt::PublicationCommitMismatch,
            BankAttempt::PublicationCommitMismatch,
        ),
        (
            Attempt::InboundOperationSlotMissing,
            BankAttempt::InboundOperationSlotMissing,
        ),
        (
            Attempt::AttemptIdentityExhausted,
            BankAttempt::AttemptIdentityExhausted,
        ),
        (
            Attempt::OutstandingDispatchMissing,
            BankAttempt::OutstandingDispatchMissing,
        ),
        (
            Attempt::OriginalPublicationPending,
            BankAttempt::OriginalPublicationPending,
        ),
        (
            Attempt::OutstandingDispatchMismatch,
            BankAttempt::OutstandingDispatchMismatch,
        ),
        (
            Attempt::InFlightCapacityExhausted,
            BankAttempt::InFlightCapacityExhausted,
        ),
    ];
    for (query, expected) in attempts {
        assert_eq!(BankAttempt::from(query), expected);
    }
}
