use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind;

use super::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

pub(in crate::http::server) fn request_mutation_denial(
    kind: WorthQueryApplicationRequestMutationDenialKind,
) -> BankHttpDenial {
    use WorthQueryApplicationRequestMutationDenialKind as Denial;
    match kind {
        Denial::ProductSelection => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Denial::ScopeResolution => BankHttpDenial::new(
            BankHttpDenialKind::NotFound,
            BankHttpNextAction::CorrectRequest,
        ),
        Denial::Authorization => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        Denial::ApplicationProgramRequired
        | Denial::ApplicationProgramMismatch
        | Denial::RequiresWorkflowTransition => BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        Denial::WorkflowAuthoritySpent | Denial::WorkflowTransitionCurrentness => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Denial::BindingInstallation
        | Denial::CapabilityInstallation
        | Denial::PrincipalResolution
        | Denial::Idempotency
        | Denial::Handler
        | Denial::SourceExpectation => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
    }
}
