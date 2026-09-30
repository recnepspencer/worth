use bank_server::{
    BankCommittedDispatchOutboxReadDenial, BankEstateProgressionDenial, BankRecoveryDenialKind,
};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

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
        D::Authorization(_)
        | D::ApprovalAuthorization(_)
        | D::CloseAuthorization(_)
        | D::ReviewAuthorization(_) => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
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
        D::Idempotency(_) | D::LifecycleProjection(_) => {
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

/// Every recovery kind chooses its own next action; none falls through a
/// wildcard, so a new kind cannot silently inherit a misleading prompt.
fn recovery_denial(kind: BankRecoveryDenialKind) -> BankHttpDenial {
    use BankRecoveryDenialKind as Kind;
    match kind {
        Kind::RecoveryNotAdmitted
        | Kind::RecoveryAlreadyMinted
        | Kind::RuntimeMismatch
        | Kind::SchemaMismatch
        | Kind::BranchMismatch
        | Kind::ApplicationBindingGenerationMismatch
        | Kind::OperationMismatch
        | Kind::GovernedInputMismatch
        | Kind::AttemptMismatch
        | Kind::PrincipalScopeMismatch
        | Kind::IdempotencyMismatch
        | Kind::ForeignIdempotencyRead
        | Kind::ProviderPostureMismatch
        | Kind::CorrelationMismatch
        | Kind::CompatibilityGenerationMismatch
        | Kind::Expired
        | Kind::AlreadyTerminal
        | Kind::ForeignPrincipal
        | Kind::ForeignRuntime
        | Kind::ForeignBranchEqualOrdinal
        | Kind::CompensationNotAdmitted
        | Kind::ReconciliationNotAdmitted
        | Kind::FreshAuthorityDenied
        | Kind::DisclosureAdmissionRequired
        | Kind::CurrentPolicyDenied => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        // The effect's completion or its evidence is still settling; the same
        // request succeeds or answers already-completed once it has.
        Kind::UnresolvedExternalPosture
        | Kind::CompletionPublicationPending
        | Kind::TerminalIndexUnavailable
        | Kind::AttemptAdmissionDenied
        | Kind::TimeObservationDenied => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
        Kind::TransportNotInstalled => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        Kind::DispatchOwnerReadDenied(read) => dispatch_owner_read_denial(read),
        // The recovered commit declared no external effect to retry.
        Kind::DispatchOutboxMissing => {
            BankHttpDenial::new(BankHttpDenialKind::NotFound, BankHttpNextAction::None)
        }
        // Safe retry answers an already-completed effect with its own outcome
        // before any denial mapping, so no route reaches this as a denial.
        Kind::AlreadyCompleted | Kind::CanonicalDerivationDenied => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    }
}

fn dispatch_owner_read_denial(read: BankCommittedDispatchOutboxReadDenial) -> BankHttpDenial {
    use BankCommittedDispatchOutboxReadDenial as Read;
    match read {
        Read::ExactCommitUnavailable => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
        Read::ActiveSnapshotCapacityExhausted { .. } | Read::SnapshotIdentityExhausted => {
            BankHttpDenial::new(
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::ContactOperator,
            )
        }
        Read::ForeignRuntime
        | Read::Missing
        | Read::WrongRecordKind
        | Read::NotAuthoritative
        | Read::Malformed
        | Read::CommitMismatch
        | Read::RecordMismatch => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    }
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
mod tests {
    use super::*;

    #[test]
    fn settling_completion_kinds_ask_to_try_again_later() {
        for kind in [
            BankRecoveryDenialKind::CompletionPublicationPending,
            BankRecoveryDenialKind::TerminalIndexUnavailable,
            BankRecoveryDenialKind::UnresolvedExternalPosture,
        ] {
            assert_eq!(
                recovery_denial(kind),
                BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn no_redispatch_refusal_prompts_a_refresh() {
        for kind in [
            BankRecoveryDenialKind::AlreadyCompleted,
            BankRecoveryDenialKind::CompletionPublicationPending,
            BankRecoveryDenialKind::TerminalIndexUnavailable,
            BankRecoveryDenialKind::DispatchOutboxMissing,
            BankRecoveryDenialKind::TransportNotInstalled,
            BankRecoveryDenialKind::DispatchOwnerReadDenied(
                BankCommittedDispatchOutboxReadDenial::ExactCommitUnavailable,
            ),
            BankRecoveryDenialKind::AttemptAdmissionDenied,
            BankRecoveryDenialKind::CanonicalDerivationDenied,
            BankRecoveryDenialKind::TimeObservationDenied,
        ] {
            assert_ne!(
                recovery_denial(kind).next_action,
                BankHttpNextAction::Refresh,
                "{kind:?} has nothing to refresh"
            );
        }
    }
}
