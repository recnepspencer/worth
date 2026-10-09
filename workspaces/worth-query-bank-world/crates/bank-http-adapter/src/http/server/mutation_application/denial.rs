use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind;

use super::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};

pub(in crate::http::server) fn request_mutation_denial(
    kind: WorthQueryApplicationRequestMutationDenialKind,
) -> BankHttpDenial {
    use WorthQueryApplicationRequestMutationDenialKind as Denial;
    match kind {
        Denial::ExecutionRequest(cause) => super::super::advancement_denial::advancement(cause),
        Denial::IdempotencyExecutionDenied(kind) => pending_execution_denial(kind),
        Denial::ProductSelection => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Denial::ScopeResolution => BankHttpDenial::new(
            BankHttpDenialKind::NotFound,
            BankHttpNextAction::CorrectRequest,
        ),
        Denial::Authorization(kind) => {
            super::super::authorization_denial::authorization_denial(kind.into())
        }
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
        // another input, the runtime serves no handler for it, the key's
        // record cannot be checked against the request, or the server's own
        // admission resolved the key against another runtime.
        Denial::Identity
        | Denial::WorkflowControl
        | Denial::InputNotAdmitted
        | Denial::HandlerNotInstalled
        | Denial::IdempotencyIntentUnverifiable
        | Denial::IdempotencyForeignAdmission => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        // The server prepared this request object once already. Every HTTP
        // request builds a fresh one, so sending it again can pass.
        Denial::PreparationSpent => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
        // No identity remains for resolving the key until reconfiguration.
        Denial::IdempotencyIdentityExhausted => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        Denial::BindingInstallation
        | Denial::CapabilityInstallation
        | Denial::PrincipalResolution
        | Denial::IdempotencyUnavailable
        | Denial::Handler
        | Denial::SourceExpectation
        | Denial::ProgramSelection => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
    }
}

// Pending publication uses the same action law as a commit refusal.
pub(in crate::http::server) fn pending_execution_denial(
    kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Kind;
    match kind {
        Kind::ExecutionResource { denial, .. } => {
            super::commit_denial::execution_resource(denial).1
        }
        Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. }
        | Kind::RetentionIdentityExhausted
        | Kind::SnapshotIdentityExhausted
        | Kind::CandidateIdentityExhausted
        | Kind::IndexGenerationIdentityExhausted
        | Kind::ForeignOperationAttempt
        | Kind::ForeignExecutionBasis
        | Kind::ForeignGraphAuthority
        | Kind::UndeclaredOperationScope
        | Kind::ResourceEnvelopeMismatch
        | Kind::ProviderIdentityMismatch
        | Kind::ProviderGenerationMismatch
        | Kind::SessionProtocolUnsupported
        | Kind::ProviderPanicked
        | Kind::TokenNotMintedForPlan
        | Kind::EmptyPhysicalSessionIdentity
        | Kind::SessionIdentityExhausted => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ActiveSnapshotCapacityExhausted { .. }
        | Kind::RetentionCapacityExhausted
        | Kind::IndexMaintenanceBudgetExceeded
        | Kind::AllocationDenied
        | Kind::ProviderRejected => {
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
    fn each_idempotency_cause_asks_for_the_action_that_can_succeed() {
        use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind as Authorization;
        let cases = [
            (
                Denial::Authorization(Authorization::DeadlineExceeded),
                BankHttpDenialKind::DeadlineExceeded,
                BankHttpNextAction::Retry,
            ),
            (
                Denial::Authorization(Authorization::ExpiredAuthentication),
                BankHttpDenialKind::Unauthenticated,
                BankHttpNextAction::Authenticate,
            ),
            (
                Denial::Authorization(Authorization::PermissionDenied),
                BankHttpDenialKind::PermissionDenied,
                BankHttpNextAction::None,
            ),
            (
                Denial::IdempotencyUnavailable,
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::Retry,
            ),
            (
                Denial::IdempotencyIdentityExhausted,
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::ContactOperator,
            ),
            (
                Denial::IdempotencyForeignAdmission,
                BankHttpDenialKind::InternalDenied,
                BankHttpNextAction::ContactOperator,
            ),
        ];
        for (kind, denial, next) in cases {
            assert_eq!(
                request_mutation_denial(kind),
                BankHttpDenial::new(denial, next),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_key_committed_before_a_restore_asks_for_a_refresh_not_a_retry() {
        assert_eq!(
            request_mutation_denial(Denial::IdempotencyReceiptNotRetained),
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        );
    }
}

#[cfg(test)]
mod pending_execution_tests;
