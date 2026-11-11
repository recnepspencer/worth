use bank_server::{
    BankCommittedDispatchOutboxReadDenial, BankEstateIdempotencyResolutionDenial,
    BankEstateProgressionDenial, BankExternalDispatchAttemptDenial, BankRecoveryDenialKind,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};
use super::authorization_denial::authorization_denial;

pub(super) fn estate_denial(denial: BankEstateProgressionDenial) -> BankHttpDenial {
    use BankEstateProgressionDenial as D;
    match denial {
        D::ProgramAction(denial) => super::mutation_application::commit_denial(denial.kind()),
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
        D::Projection(bank_server::BankEstateOperationProjectionDenial::Authorization(cause))
        | D::Projection(bank_server::BankEstateOperationProjectionDenial::AuthorizationLineageUnavailable(cause)) => authorization_denial(cause.kind()),
        D::Projection(bank_server::BankEstateOperationProjectionDenial::InvariantAdmission(
            worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionDenialKind::ExecutionRequest(cause),
        )) => super::relational_execution_denial::denial(cause),
        D::Attempt(denial) => match *denial {
            bank_server::BankCommitPreparationDenial::StorageUnavailable(failure) => super::relational_execution_denial::denial(worth_query_host::facade::primary_graph::RelationalExecutionDenialCause::StorageUnavailable(failure)),
            bank_server::BankCommitPreparationDenial::Application { kind, .. } => attempt_kind(kind),
        },
        D::CommandInput(_)
        | D::Projection(_)
        | D::DecisionProjection(_)
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
        | D::OperationInstallation(_) => {
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
        Idempotency::OriginalEffectReleased => {
            (Denial::OriginalEffectReleased, Next::ContactOperator)
        }
        Idempotency::PreparationBusy => (Denial::Unavailable, Next::Retry),
        Idempotency::ExecutionDenied(kind) => {
            return super::mutation_application::pending_execution_stop(kind)
        }
        Idempotency::Authorization(denial) => return authorization_denial(denial.kind()),
        // The key's earlier commit took effect; reading current state shows it.
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
        Kind::AdmissionAuthorization(cause) => return authorization_denial(cause),
        Kind::CompletionExecutionDenied { kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::Content(cause), .. } => return super::relational_execution_denial::denial(worth_query_host::facade::primary_graph::RelationalExecutionDenialCause::Content(cause)),
        Kind::CompletionExecutionDenied { kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::ContentInconsistency, .. } => (Denial::InternalDenied, Next::ContactOperator),
        Kind::CompletionExecutionRequest(cause) => {
            return super::advancement_denial::advancement(cause)
        }
        // Durable custody support is a runtime configuration fact, not a caller retry.
        Kind::DurableStateAdmission(cause) => match cause {
            worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::Unsupported { .. }
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionRelationPreimage
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionCoupledOutputs
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionCompensation
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionOutputMaterial
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedCorrectionReapplication
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UnsupportedDivergentCorrection => (Denial::Unavailable, Next::ContactOperator),
            worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::PolicyUnavailable
            | worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UndeclaredSupplierCapabilities => (Denial::InternalDenied, Next::ContactOperator),
        },
        // The installed contract refuses this recovery, or the principal is not
        // the one it belongs to. No retry, refresh, or new credential changes
        // that.
        Kind::RecoveryNotAdmitted
        | Kind::CompensationNotAdmitted
        | Kind::ReconciliationNotAdmitted
        | Kind::ForeignPrincipal => (Denial::PermissionDenied, Next::None),
        // The recovery already moved on: it was opened, or it ended. Reading
        // the commit again reports its current recovery status.
        Kind::StaleIncarnation | Kind::RecoveryAlreadyMinted | Kind::AlreadyTerminal => (Denial::Stale, Next::Refresh),
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
        | Kind::CompletionExecutionDenied { .. }
        | Kind::CompletionExecutionControlStopped { .. }
        | Kind::CompletionAllocationDenied { .. }
        | Kind::CompletionStagingCardinalityOverflow
        | Kind::CompletionInputDirectoryAllocationDenied { .. }
        | Kind::TerminalIndexUnavailable
        | Kind::TimeObservationDenied => (Denial::Unavailable, Next::Retry),
        Kind::ProvenNoEffect(cause) => match cause {
            // The original payload is immutable: repeating this recovery request
            // cannot correct the external owner's semantic rejection.
            worth_foundational::facade::ExternalEffectNoEffectCause::PayloadRejected => (Denial::Unavailable, Next::ContactOperator),
            worth_foundational::facade::ExternalEffectNoEffectCause::ContactNotStarted => (Denial::Unavailable, Next::Retry),
        },
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
        Read::ReadExecution(Ok(cause)) => return super::mutation_application::pending_execution_denial(cause),
        Read::ReadExecution(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled)) => (Denial::Cancelled, Next::Retry),
        Read::ReadExecution(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut)) => (Denial::DeadlineExceeded, Next::Retry),

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
        Read::ForeignPhase
        | Read::ForeignRuntime
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
    use bank_server::BankDurableDispatchRecordDenial as Record;
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
        // Nothing was sent. An earlier send is still being observed, the
        // request stopped, or the record was proven unwritten: the same
        // request records and sends once that clears.
        Attempt::DurableAttemptRecordDenied(
            Record::AttemptAwaitingObservation
            | Record::RequestStopped
            | Record::PublicationNoEffect,
        ) => (Denial::Unavailable, Next::Retry),
        // The record's fate, or the journal itself, needs the Home reopened.
        Attempt::DurableAttemptRecordDenied(
            Record::PublicationUnresolved | Record::JournalRefused,
        ) => (Denial::Unavailable, Next::ContactOperator),
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
        Query::Refused(PrincipalRefusal::ExpiredAuthentication) => BankHttpDenial::new(
            BankHttpDenialKind::Unauthenticated,
            BankHttpNextAction::Authenticate,
        ),
        Query::Refused(PrincipalRefusal::StaleInstalledSchema)
        | Query::Refused(PrincipalRefusal::StalePrincipalProof)
        | Query::Refused(PrincipalRefusal::BranchMaterializationSuspended) => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Query::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled)) => {
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
        }
        Query::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut)) => BankHttpDenial::new(
            BankHttpDenialKind::DeadlineExceeded,
            BankHttpNextAction::Retry,
        ),
        Query::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
        | Query::Refused(PrincipalRefusal::BindingNotInstalled)
        | Query::Refused(PrincipalRefusal::IdentityIndexUnavailable)
        | Query::Refused(PrincipalRefusal::CorruptIdentityIndex)
        | Query::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted{ .. })
        | Query::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
        | Query::Refused(PrincipalRefusal::RetentionCapacityExhausted)
        | Query::Refused(PrincipalRefusal::RetentionIdentityExhausted) => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        Query::Refused(PrincipalRefusal::ForeignRuntime)
        | Query::Refused(PrincipalRefusal::UnknownPrincipal)
        | Query::Refused(PrincipalRefusal::DisabledPrincipal)
        | Query::Refused(PrincipalRefusal::AmbiguousPrincipal)
        | Query::Refused(PrincipalRefusal::MissingPrincipalTarget)
        | Query::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
        | Query::Refused(PrincipalRefusal::WrongPrincipalTargetKind) => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        Query::Stopped(Ok(_)) | Query::Refused(PrincipalRefusal::ForeignPhase) => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    }
}

#[cfg(test)]
#[path = "estate_denial/recovery_instruction_tests.rs"]
mod recovery_instruction_tests;

pub(in crate::http::server) fn attempt_kind(
    kind: bank_server::BankApplicationAttemptDenialKind,
) -> BankHttpDenial {
    use bank_server::BankApplicationAttemptDenialKind as Attempt;
    match kind {
        Attempt::Cancelled => {
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
        }
        Attempt::DeadlineExceeded => BankHttpDenial::new(
            BankHttpDenialKind::DeadlineExceeded,
            BankHttpNextAction::Retry,
        ),
        Attempt::ExecutionRequest(cause) => {
            super::mutation_application::pending_execution_denial(cause)
        }
        Attempt::EffectExecution(cause) => super::advancement_denial::request(cause),
        Attempt::ValueDecode(cause) => super::query_denial::value_decode(cause),
        Attempt::ContentInconsistency => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        _ => BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
    }
}
