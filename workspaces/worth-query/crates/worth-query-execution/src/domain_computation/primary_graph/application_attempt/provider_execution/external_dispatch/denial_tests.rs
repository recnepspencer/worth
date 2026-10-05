//! Each runtime refusal of a physical attempt keeps its own preparation kind.

use std::collections::BTreeSet;

use super::{
    from_attempt_admission, from_in_flight, redispatch_preparation,
    WorthQueryExternalDispatchAttemptDenial as Attempt,
    WorthQueryExternalDispatchPreparationDenial as Preparation,
    WorthQueryExternalRedispatchDenial as Redispatch,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryExternalDispatchAdmissionDenial as Admission;
use crate::domain_computation::primary_graph::provider::WorthQueryOutstandingInFlightDenial as InFlight;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial as Read;

const ADMISSIONS: [Admission; 8] = [
    Admission::ForeignRelationalRuntime,
    Admission::ForeignProductWorld,
    Admission::PublicationCommitMismatch,
    Admission::MissingInboundOperationSlot,
    Admission::AlreadyCompleted,
    Admission::CompletedTransportRetained,
    Admission::TerminalIndexUnavailable,
    Admission::AttemptIdentityExhausted,
];

const IN_FLIGHT: [InFlight; 5] = [
    InFlight::Missing,
    InFlight::PendingPublication,
    InFlight::OriginalMismatch,
    InFlight::TerminalReached,
    InFlight::CapacityExhausted,
];

#[test]
fn every_attempt_refusal_names_its_own_preparation_kind() {
    let refusals = ADMISSIONS
        .map(from_attempt_admission)
        .into_iter()
        .chain(IN_FLIGHT.map(from_in_flight))
        .collect::<Vec<_>>();
    let kinds = refusals
        .iter()
        .map(|denial| format!("{denial:?}"))
        .collect::<BTreeSet<_>>();
    // A World terminal reached after admission is the same completed effect
    // the terminal owner reports at admission; every other cause is its own.
    assert_eq!(
        kinds.len(),
        ADMISSIONS.len() + IN_FLIGHT.len() - 1,
        "two attempt refusals share one preparation kind: {refusals:?}"
    );
}

#[test]
fn attempt_refusals_name_the_exact_cause() {
    let expected = [
        (
            from_attempt_admission(Admission::ForeignRelationalRuntime),
            Attempt::ForeignRelationalRuntime,
        ),
        (
            from_attempt_admission(Admission::ForeignProductWorld),
            Attempt::ForeignProductWorld,
        ),
        (
            from_attempt_admission(Admission::PublicationCommitMismatch),
            Attempt::PublicationCommitMismatch,
        ),
        (
            from_attempt_admission(Admission::MissingInboundOperationSlot),
            Attempt::InboundOperationSlotMissing,
        ),
        (
            from_attempt_admission(Admission::AttemptIdentityExhausted),
            Attempt::AttemptIdentityExhausted,
        ),
        (
            from_in_flight(InFlight::Missing),
            Attempt::OutstandingDispatchMissing,
        ),
        (
            from_in_flight(InFlight::PendingPublication),
            Attempt::OriginalPublicationPending,
        ),
        (
            from_in_flight(InFlight::OriginalMismatch),
            Attempt::OutstandingDispatchMismatch,
        ),
        (
            from_in_flight(InFlight::CapacityExhausted),
            Attempt::InFlightCapacityExhausted,
        ),
    ];
    for (mapped, attempt) in expected {
        assert_eq!(mapped, Preparation::AttemptAdmissionDenied(attempt));
    }
    assert_eq!(
        from_attempt_admission(Admission::AlreadyCompleted),
        Preparation::AlreadyCompleted
    );
    assert_eq!(
        from_in_flight(InFlight::TerminalReached),
        Preparation::AlreadyCompleted
    );
    assert_eq!(
        from_attempt_admission(Admission::CompletedTransportRetained),
        Preparation::CompletionPublicationPending
    );
    assert_eq!(
        from_attempt_admission(Admission::TerminalIndexUnavailable),
        Preparation::TerminalIndexUnavailable
    );
}

#[test]
fn redispatch_keeps_every_preparation_cause() {
    for attempt in super::super::redispatch_denial_tests::ATTEMPTS {
        assert_eq!(
            redispatch_preparation(Preparation::AttemptAdmissionDenied(attempt)),
            Redispatch::AttemptAdmissionDenied(attempt)
        );
    }
    assert_eq!(
        redispatch_preparation(Preparation::OwnerReadDenied(Read::Missing)),
        Redispatch::OwnerReadDenied(Read::Missing)
    );
    assert_eq!(
        redispatch_preparation(Preparation::TimeObservationDenied),
        Redispatch::TimeObservationDenied
    );
    assert_eq!(
        redispatch_preparation(Preparation::CanonicalDerivationDenied),
        Redispatch::CanonicalDerivationDenied
    );
}
