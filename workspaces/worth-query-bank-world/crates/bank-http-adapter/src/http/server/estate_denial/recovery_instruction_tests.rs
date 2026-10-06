//! Each recovery refusal tells the caller the one action that can succeed.

use super::*;
use BankCommittedDispatchOutboxReadDenial as Read;
use BankExternalDispatchAttemptDenial as Attempt;
use BankHttpDenialKind as Denial;
use BankHttpNextAction as Next;
use BankRecoveryDenialKind as Kind;

const OWNER_READS: [(Read, Denial, Next); 13] = [
    (Read::PendingPublication, Denial::Unavailable, Next::Retry),
    (
        Read::CommittedIndexUnavailable,
        Denial::Unavailable,
        Next::Retry,
    ),
    (
        Read::ExactCommitUnavailable,
        Denial::Unavailable,
        Next::ContactOperator,
    ),
    (
        Read::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 3,
        },
        Denial::Unavailable,
        Next::Retry,
    ),
    (
        Read::SnapshotIdentityExhausted,
        Denial::Unavailable,
        Next::ContactOperator,
    ),
    (
        Read::ForeignRuntime,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (Read::Missing, Denial::InternalDenied, Next::ContactOperator),
    (
        Read::AmbiguousCorrelation,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Read::WrongRecordKind,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Read::NotAuthoritative,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Read::Malformed,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Read::CommitMismatch,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Read::RecordMismatch,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
];

const ATTEMPTS: [(Attempt, Denial, Next); 9] = [
    (
        Attempt::OriginalPublicationPending,
        Denial::Unavailable,
        Next::Retry,
    ),
    (
        Attempt::InFlightCapacityExhausted,
        Denial::Unavailable,
        Next::Retry,
    ),
    (
        Attempt::AttemptIdentityExhausted,
        Denial::Unavailable,
        Next::ContactOperator,
    ),
    (
        Attempt::InboundOperationSlotMissing,
        Denial::Stale,
        Next::ContactOperator,
    ),
    (
        Attempt::ForeignRelationalRuntime,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Attempt::ForeignProductWorld,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Attempt::PublicationCommitMismatch,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Attempt::OutstandingDispatchMissing,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
    (
        Attempt::OutstandingDispatchMismatch,
        Denial::InternalDenied,
        Next::ContactOperator,
    ),
];

/// Every recovery kind with the instruction it must carry. The exhaustive
/// match stops compiling when a kind is added, so a new kind cannot skip this
/// table.
fn every_kind() -> Vec<(Kind, Denial, Next)> {
    let listed = |kind: Kind| match kind {
        Kind::RecoveryNotAdmitted
        | Kind::RecoveryAlreadyMinted
        | Kind::SchemaMismatch
        | Kind::BranchMismatch
        | Kind::ApplicationBindingGenerationMismatch
        | Kind::OperationMismatch
        | Kind::GovernedInputMismatch
        | Kind::ForeignIdempotencyRead
        | Kind::CorrelationMismatch
        | Kind::CompatibilityGenerationMismatch
        | Kind::Expired
        | Kind::AlreadyTerminal
        | Kind::ForeignPrincipal
        | Kind::ForeignRuntime
        | Kind::ForeignBranchEqualOrdinal
        | Kind::AlreadyCompleted
        | Kind::CompletionPublicationPending
        | Kind::TerminalIndexUnavailable
        | Kind::DispatchOutboxMissing
        | Kind::TransportNotInstalled
        | Kind::DispatchOwnerReadDenied(_)
        | Kind::AttemptAdmissionDenied(_)
        | Kind::CanonicalDerivationDenied
        | Kind::TimeObservationDenied
        | Kind::CompensationNotAdmitted
        | Kind::ReconciliationNotAdmitted
        | Kind::FreshAuthorityDenied
        | Kind::AdmissionCancelled
        | Kind::AdmissionDeadlineExceeded
        | Kind::AdmissionAuthenticationExpired
        | Kind::DisclosureAdmissionRequired
        | Kind::UnresolvedExternalPosture => kind,
    };
    let refused = (Denial::PermissionDenied, Next::None);
    let moved_on = (Denial::Stale, Next::Refresh);
    let unrecoverable = (Denial::Stale, Next::ContactOperator);
    let settling = (Denial::Unavailable, Next::Retry);
    let server_fault = (Denial::InternalDenied, Next::ContactOperator);
    [
        (Kind::RecoveryNotAdmitted, refused),
        (Kind::CompensationNotAdmitted, refused),
        (Kind::ReconciliationNotAdmitted, refused),
        (Kind::ForeignPrincipal, refused),
        (Kind::RecoveryAlreadyMinted, moved_on),
        (Kind::AlreadyTerminal, moved_on),
        (Kind::Expired, unrecoverable),
        (Kind::SchemaMismatch, unrecoverable),
        (Kind::BranchMismatch, unrecoverable),
        (Kind::ForeignBranchEqualOrdinal, unrecoverable),
        (Kind::ApplicationBindingGenerationMismatch, unrecoverable),
        (Kind::OperationMismatch, unrecoverable),
        (Kind::GovernedInputMismatch, unrecoverable),
        (Kind::CompatibilityGenerationMismatch, unrecoverable),
        (Kind::AdmissionCancelled, (Denial::Cancelled, Next::Retry)),
        (
            Kind::AdmissionDeadlineExceeded,
            (Denial::DeadlineExceeded, Next::Retry),
        ),
        (
            Kind::AdmissionAuthenticationExpired,
            (Denial::Unauthenticated, Next::Authenticate),
        ),
        (Kind::UnresolvedExternalPosture, settling),
        (Kind::CompletionPublicationPending, settling),
        (Kind::TerminalIndexUnavailable, settling),
        (Kind::TimeObservationDenied, settling),
        (
            Kind::TransportNotInstalled,
            (Denial::Unavailable, Next::ContactOperator),
        ),
        (Kind::DispatchOutboxMissing, (Denial::NotFound, Next::None)),
        (Kind::ForeignRuntime, server_fault),
        (Kind::ForeignIdempotencyRead, server_fault),
        (Kind::CorrelationMismatch, server_fault),
        (Kind::FreshAuthorityDenied, server_fault),
        (Kind::DisclosureAdmissionRequired, server_fault),
        (Kind::AlreadyCompleted, server_fault),
        (Kind::CanonicalDerivationDenied, server_fault),
    ]
    .into_iter()
    .map(|(kind, (denial, next))| (listed(kind), denial, next))
    .chain(
        OWNER_READS.into_iter().map(|(read, denial, next)| {
            (listed(Kind::DispatchOwnerReadDenied(read)), denial, next)
        }),
    )
    .chain(ATTEMPTS.into_iter().map(|(attempt, denial, next)| {
        (listed(Kind::AttemptAdmissionDenied(attempt)), denial, next)
    }))
    .collect()
}

#[test]
fn every_recovery_kind_names_the_action_that_can_succeed() {
    for (kind, denial, next) in every_kind() {
        assert_eq!(
            recovery_denial(kind),
            BankHttpDenial::new(denial, next),
            "{kind:?}"
        );
    }
}

#[test]
fn a_contract_refusal_asks_for_nothing_it_cannot_change() {
    for kind in [
        Kind::RecoveryNotAdmitted,
        Kind::CompensationNotAdmitted,
        Kind::ReconciliationNotAdmitted,
    ] {
        assert_eq!(
            recovery_denial(kind),
            BankHttpDenial::new(Denial::PermissionDenied, Next::None),
            "{kind:?} is the installed contract's answer, not a stale view"
        );
    }
}

#[test]
fn a_lapsed_admission_and_a_foreign_authority_ask_for_different_actions() {
    let fresh = recovery_denial(Kind::FreshAuthorityDenied);
    for (lapse, cleared_by) in [
        (Kind::AdmissionCancelled, Next::Retry),
        (Kind::AdmissionDeadlineExceeded, Next::Retry),
        (Kind::AdmissionAuthenticationExpired, Next::Authenticate),
    ] {
        let lapsed = recovery_denial(lapse);
        assert_eq!(lapsed.next_action, cleared_by, "{lapse:?}");
        assert_ne!(lapsed, fresh, "{lapse:?} is not a fresh-authority failure");
    }
}

#[test]
fn a_settling_owner_read_is_retried_but_lost_history_and_spent_identity_are_not() {
    let owner_read = |read| recovery_denial(Kind::DispatchOwnerReadDenied(read));
    assert_eq!(
        owner_read(Read::PendingPublication).next_action,
        Next::Retry
    );
    assert_eq!(
        owner_read(Read::CommittedIndexUnavailable).next_action,
        Next::Retry
    );
    assert_eq!(
        owner_read(Read::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 3,
        })
        .next_action,
        Next::Retry,
        "concurrent reads can release the occupied snapshot capacity"
    );
    assert_eq!(
        owner_read(Read::SnapshotIdentityExhausted).next_action,
        Next::ContactOperator,
        "releasing a snapshot cannot replenish spent identities"
    );
    assert_eq!(
        owner_read(Read::ExactCommitUnavailable).next_action,
        Next::ContactOperator,
        "a commit whose exact version is gone never reads back"
    );
}

#[test]
fn a_full_send_window_is_retried_but_a_foreign_original_is_not() {
    let attempt = |attempt| recovery_denial(Kind::AttemptAdmissionDenied(attempt));
    assert_eq!(
        attempt(Attempt::InFlightCapacityExhausted).next_action,
        Next::Retry
    );
    assert_eq!(
        attempt(Attempt::ForeignProductWorld).next_action,
        Next::ContactOperator,
        "no retry changes whose world the original belongs to"
    );
}

#[test]
fn no_redispatch_refusal_prompts_a_refresh() {
    for kind in [
        Kind::AlreadyCompleted,
        Kind::CompletionPublicationPending,
        Kind::TerminalIndexUnavailable,
        Kind::DispatchOutboxMissing,
        Kind::TransportNotInstalled,
        Kind::CanonicalDerivationDenied,
        Kind::TimeObservationDenied,
        Kind::AdmissionCancelled,
        Kind::AdmissionDeadlineExceeded,
        Kind::AdmissionAuthenticationExpired,
    ]
    .into_iter()
    .chain(OWNER_READS.map(|(read, _, _)| Kind::DispatchOwnerReadDenied(read)))
    .chain(ATTEMPTS.map(|(attempt, _, _)| Kind::AttemptAdmissionDenied(attempt)))
    {
        assert_ne!(
            recovery_denial(kind).next_action,
            Next::Refresh,
            "{kind:?} has nothing to refresh"
        );
    }
}

#[test]
fn a_committed_key_asks_for_a_refresh_and_an_unverifiable_one_for_the_operator() {
    use bank_server::BankEstateIdempotencyResolutionDenial as Idempotency;
    assert_eq!(
        estate_denial(BankEstateProgressionDenial::Idempotency(
            Idempotency::CommittedReceiptNotRetained
        )),
        BankHttpDenial::new(Denial::Stale, Next::Refresh)
    );
    assert_eq!(
        estate_denial(BankEstateProgressionDenial::Idempotency(
            Idempotency::RecordedIntentUnverifiable
        )),
        BankHttpDenial::new(Denial::InternalDenied, Next::ContactOperator)
    );
}

#[test]
fn idempotency_capacity_is_retried_and_spent_identity_asks_for_the_operator() {
    use bank_server::BankEstateIdempotencyResolutionDenial as Idempotency;
    for (cause, denial, next) in [
        (
            Idempotency::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 2,
            },
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::RetentionCapacityExhausted,
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::ProviderUnavailable,
            Denial::Unavailable,
            Next::Retry,
        ),
        (
            Idempotency::RetentionIdentityExhausted,
            Denial::Unavailable,
            Next::ContactOperator,
        ),
        (
            Idempotency::SnapshotIdentityExhausted,
            Denial::Unavailable,
            Next::ContactOperator,
        ),
        (
            Idempotency::ForeignAdmission,
            Denial::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        assert_eq!(
            estate_denial(BankEstateProgressionDenial::Idempotency(cause)),
            BankHttpDenial::new(denial, next),
            "{cause:?}"
        );
    }
}
