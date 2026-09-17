use bank_server::{BankEstateProgressionDenial, BankRecoveryDenialKind};
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;

use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

pub(super) fn estate_denial(denial: BankEstateProgressionDenial) -> BankHttpDenial {
    use BankEstateProgressionDenial as D;
    match denial {
        D::ProgramAction(denial) => super::mutation_application::commit_denial(denial.kind()).1,
        D::ProgramMismatch
        | D::PrincipalBindingInstallation(_)
        | D::PrincipalIdentityEncoding(_) => BankHttpDenial::new(
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
        D::Recovery(denial)
            if denial.kind() == BankRecoveryDenialKind::UnresolvedExternalPosture =>
        {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
        D::Recovery(_) | D::Idempotency(_) | D::LifecycleProjection(_) => {
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
