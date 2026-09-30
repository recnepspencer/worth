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
        // State already moved on, by a later workflow step or by the key's
        // earlier commit; reading current state shows it.
        Denial::WorkflowAuthoritySpent
        | Denial::WorkflowTransitionCurrentness
        | Denial::IdempotencyReceiptNotRetained => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        // The same request can never pass these, so a retry would loop forever:
        // the binding needs a workflow transition, the admission governed
        // another input, the runtime serves no handler for it, or the key's
        // record cannot be checked against the request.
        Denial::Identity
        | Denial::WorkflowControl
        | Denial::InputNotAdmitted
        | Denial::HandlerNotInstalled
        | Denial::IdempotencyIntentUnverifiable => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        Denial::BindingInstallation
        | Denial::CapabilityInstallation
        | Denial::PrincipalResolution
        | Denial::Idempotency
        | Denial::Handler
        | Denial::SourceExpectation
        | Denial::ProgramSelection => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind as Denial;

    use super::{request_mutation_denial, BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

    #[test]
    fn handler_refusals_no_retry_can_pass_ask_for_the_operator() {
        let operator = BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        );
        for kind in [
            Denial::WorkflowControl,
            Denial::InputNotAdmitted,
            Denial::HandlerNotInstalled,
            Denial::Identity,
            Denial::IdempotencyIntentUnverifiable,
        ] {
            assert_eq!(request_mutation_denial(kind), operator, "{kind:?}");
        }
        assert_eq!(
            request_mutation_denial(Denial::Handler),
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        );
    }

    #[test]
    fn a_key_committed_before_a_restore_asks_for_a_refresh_not_a_retry() {
        assert_eq!(
            request_mutation_denial(Denial::IdempotencyReceiptNotRetained),
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        );
    }
}
