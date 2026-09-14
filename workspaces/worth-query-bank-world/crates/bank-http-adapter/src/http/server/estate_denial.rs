use bank_server::BankEstateProgressionDenial;

use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

pub(super) fn estate_denial(denial: BankEstateProgressionDenial) -> BankHttpDenial {
    use BankEstateProgressionDenial as D;
    match denial {
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
