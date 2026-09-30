use bank_server::{
    BankCommittedDispatchOutboxReadDenial, BankEstateIdempotencyResolutionDenial,
    BankEstateProgressionDenial, BankExternalDispatchAttemptDenial, BankRecoveryDenialKind,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};
use super::authorization_denial::authorization_denial;

pub(super) fn estate_denial(denial: BankEstateProgressionDenial) -> BankHttpDenial {
    use BankEstateProgressionDenial as D;
    match denial {
        D::ProgramAction(denial) => super::mutation_application::commit_denial(denial.kind()).1,
        D::ProgramMismatch
        | D::PrincipalBindingInstallation(_)
        | D::IdentityEncoding(_)
        | D::MutationIdentityEncoding(_) => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        D::PrincipalResolution(denial) => principal_resolution_denial(denial.kind()),
        D::ApplicationEntry(denial) => {
            super::mutation_application::request_mutation_denial(denial.kind())
        }
        D::ProductSelection(_) => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        D::Authorization(denial)
        | D::ApprovalAuthorization(denial)
        | D::CloseAuthorization(denial)
        | D::ReviewAuthorization(denial) => authorization_denial(denial.kind()),
        D::CommandInput(_)
        | D::Projection(_)
        | D::DecisionProjection(_)
        | D::FreezeProjection(_)
        | D::DeathNotificationProjection(_)
        | D::CaseOpeningProjection(_)
        | D::ExecutorRecognitionProjection(_)
        | D::EstateReleaseProjection(_)
        | D::EstateDisbursementProjection(_)
        | D::Proposal(_)
        | D::CapabilityDelegationProjection(_)
        | D::CapabilityRevocationProjection(_) => BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        D::IdempotencyIntentDrift => BankHttpDenial::new(
            BankHttpDenialKind::Stale,
            BankHttpNextAction::CorrectRequest,
        ),
        D::Recovery(denial) => recovery_denial(denial.kind()),
        D::Idempotency(denial) => idempotency_denial(denial),
        D::LifecycleProjection(_) => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        D::CapabilityInstallation(_)
        | D::OperationInstallation(_)
        | D::CommitPreparation(_)
        | D::Attempt(_) => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
    }
}

/// Every idempotency cause chooses the one action that can succeed for it.
fn idempotency_denial(denial: BankEstateIdempotencyResolutionDenial) -> BankHttpDenial {
    use BankEstateIdempotencyResolutionDenial as Idempotency;
    use BankHttpDenialKind as Denial;
    use BankHttpNextAction as Next;
    let (denial, next) = match denial {
        Idempotency::Authorization(denial) => return authorization_denial(denial.kind()),
        // The key's earlier commit took effect; reading current state shows it.
        Idempotency::CommittedReceiptNotRetained => (Denial::Stale, Next::Refresh),
        // Capacity in use frees as other reads settle.
        Idempotency::ActiveSnapshotCapacityExhausted { .. }
        | Idempotency::RetentionCapacityExhausted
        | Idempotency::ProviderUnavailable => (Denial::Unavailable, Next::Retry),
        // Identity space never frees without reconfiguration.
        Idempotency::RetentionIdentityExhausted | Idempotency::SnapshotIdentityExhausted => {
            (Denial::Unavailable, Next::ContactOperator)
        }
        // The server admitted the request itself, so a foreign admission is a
        // server fault, and an unverifiable record never becomes checkable.
        Idempotency::ForeignAdmission | Idempotency::RecordedIntentUnverifiable => {
            (Denial::InternalDenied, Next::ContactOperator)
        }
    };
    BankHttpDenial::new(denial, next)
}

/// Every recovery kind chooses its own next action; none falls through a
/// wildcard, so a new kind cannot silently inherit a misleading prompt. Each
/// instruction is the one action that can succeed for that cause.
fn recovery_denial(kind: BankRecoveryDenialKind) -> BankHttpDenial {
    use BankHttpDenialKind as Denial;
    use BankHttpNextAction as Next;
    use BankRecoveryDenialKind as Kind;
    let (denial, next) = match kind {
        // The installed contract refuses this recovery, or the principal is not
        // the one it belongs to. No retry, refresh, or new credential changes
        // that.
        Kind::RecoveryNotAdmitted
        | Kind::CompensationNotAdmitted
        | Kind::ReconciliationNotAdmitted
        | Kind::ForeignPrincipal => (Denial::PermissionDenied, Next::None),
        // The recovery already moved on: it was opened, or it ended. Reading
        // the commit again reports its current recovery status.
        Kind::RecoveryAlreadyMinted | Kind::AlreadyTerminal => (Denial::Stale, Next::Refresh),
        // The live recovery can never be admitted again: its window closed, or
        // installed truth moved past the commit it was bound to. The effect
        // stays unsettled, so an operator settles it.
        Kind::Expired
        | Kind::SchemaMismatch
        | Kind::BranchMismatch
        | Kind::ForeignBranchEqualOrdinal
        | Kind::ApplicationBindingGenerationMismatch
        | Kind::OperationMismatch
        | Kind::GovernedInputMismatch
        | Kind::CompatibilityGenerationMismatch => (Denial::Stale, Next::ContactOperator),
        // The admitted request lapsed before the recovery took effect. The
        // same request with fresh request authority can succeed.
        Kind::AdmissionCancelled => (Denial::Cancelled, Next::Retry),
        Kind::AdmissionDeadlineExceeded => (Denial::DeadlineExceeded, Next::Retry),
        Kind::AdmissionAuthenticationExpired => (Denial::Unauthenticated, Next::Authenticate),
        // The effect's completion or its evidence is still settling; the same
        // request succeeds or answers already-completed once it has.
        Kind::UnresolvedExternalPosture
        | Kind::CompletionPublicationPending
        | Kind::TerminalIndexUnavailable
        | Kind::TimeObservationDenied => (Denial::Unavailable, Next::Retry),
        Kind::TransportNotInstalled => (Denial::Unavailable, Next::ContactOperator),
        Kind::DispatchOwnerReadDenied(read) => return dispatch_owner_read_denial(read),
        Kind::AttemptAdmissionDenied(attempt) => return dispatch_attempt_denial(attempt),
        // The recovered commit declared no external effect to retry.
        Kind::DispatchOutboxMissing => (Denial::NotFound, Next::None),
        // The server holds the handle and mints both the admission and the
        // authority it checks against, so a disagreement between them is a
        // server fault. Safe retry answers an already-completed effect with
        // its own outcome before any denial mapping, so no route reaches that
        // kind as a denial either.
        Kind::ForeignRuntime
        | Kind::ForeignIdempotencyRead
        | Kind::CorrelationMismatch
        | Kind::FreshAuthorityDenied
        | Kind::DisclosureAdmissionRequired
        | Kind::AlreadyCompleted
        | Kind::CanonicalDerivationDenied => (Denial::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(denial, next)
}

fn dispatch_owner_read_denial(read: BankCommittedDispatchOutboxReadDenial) -> BankHttpDenial {
    use BankCommittedDispatchOutboxReadDenial as Read;
    use BankHttpDenialKind as Denial;
    use BankHttpNextAction as Next;
    let (denial, next) = match read {
        // Publication, indexing or concurrent snapshot occupancy can settle
        // before the same owner read is retried.
        Read::PendingPublication
        | Read::CommittedIndexUnavailable
        | Read::ActiveSnapshotCapacityExhausted { .. } => (Denial::Unavailable, Next::Retry),
        // The exact committed version or its retained basis is gone, so no
        // later read recovers it. Spent snapshot identities also require
        // operator repair, unlike capacity occupied by concurrent reads.
        Read::ExactCommitUnavailable | Read::SnapshotIdentityExhausted => {
            (Denial::Unavailable, Next::ContactOperator)
        }
        Read::ForeignRuntime
        | Read::Missing
        | Read::AmbiguousCorrelation
        | Read::WrongRecordKind
        | Read::NotAuthoritative
        | Read::Malformed
        | Read::CommitMismatch
        | Read::RecordMismatch => (Denial::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(denial, next)
}

fn dispatch_attempt_denial(attempt: BankExternalDispatchAttemptDenial) -> BankHttpDenial {
    use BankExternalDispatchAttemptDenial as Attempt;
    use BankHttpDenialKind as Denial;
    use BankHttpNextAction as Next;
    let (denial, next) = match attempt {
        // The original is still publishing, or every permitted send for this
        // effect is on the wire. Both clear on their own; the same request
        // then sends or answers already-completed.
        Attempt::OriginalPublicationPending | Attempt::InFlightCapacityExhausted => {
            (Denial::Unavailable, Next::Retry)
        }
        // No physical attempt identity remains on this runtime.
        Attempt::AttemptIdentityExhausted => (Denial::Unavailable, Next::ContactOperator),
        // A restored legacy row lacks the slot its completion needs, so no
        // physical send may start; an operator settles the effect.
        Attempt::InboundOperationSlotMissing => (Denial::Stale, Next::ContactOperator),
        // The server read the committed original itself, so a foreign or
        // disagreeing original is a server fault.
        Attempt::ForeignRelationalRuntime
        | Attempt::ForeignProductWorld
        | Attempt::PublicationCommitMismatch
        | Attempt::OutstandingDispatchMissing
        | Attempt::OutstandingDispatchMismatch => (Denial::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(denial, next)
}

fn principal_resolution_denial(kind: WorthQueryPrincipalResolutionDenialKind) -> BankHttpDenial {
    use WorthQueryPrincipalResolutionDenialKind as Query;
    match kind {
        Query::ExpiredAuthentication => BankHttpDenial::new(
            BankHttpDenialKind::Unauthenticated,
            BankHttpNextAction::Authenticate,
        ),
        Query::StaleInstalledSchema
        | Query::StalePrincipalProof
        | Query::BranchMaterializationSuspended => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Query::Cancelled => {
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
        }
        Query::DeadlineExceeded => BankHttpDenial::new(
            BankHttpDenialKind::DeadlineExceeded,
            BankHttpNextAction::Retry,
        ),
        Query::PrimaryGraphNotInstalled
        | Query::BindingNotInstalled
        | Query::IdentityIndexUnavailable
        | Query::CorruptIdentityIndex
        | Query::ActiveSnapshotCapacityExhausted { .. }
        | Query::SnapshotIdentityExhausted
        | Query::RetentionCapacityExhausted
        | Query::RetentionIdentityExhausted => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        Query::ForeignRuntime
        | Query::UnknownPrincipal
        | Query::DisabledPrincipal
        | Query::AmbiguousPrincipal
        | Query::MissingPrincipalTarget
        | Query::AmbiguousPrincipalTarget
        | Query::WrongPrincipalTargetKind => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        _ => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    }
}

#[cfg(test)]
#[path = "estate_denial/recovery_instruction_tests.rs"]
mod recovery_instruction_tests;
