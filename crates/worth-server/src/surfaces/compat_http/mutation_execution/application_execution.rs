//! Keyed requests enter the real application before any fresh-basis checks.
use super::{
    WorthServerAdmittedQueryMutation, WorthServerCompatibilityMutationExecution as Execution,
    WorthServerCompatibilityMutationOutcome, WorthServerCompatibilityMutationRequest,
    WorthServerIdempotencyKey,
};
use crate::{
    WorthServerCompatibilityFacade, WorthServerCompatibilityPreparedRequest,
    WorthServerOperationRequest, WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
};
use worth_proof::TransitionOutcome;

pub(super) fn execute(
    facade: &WorthServerCompatibilityFacade,
    prepared: WorthServerCompatibilityPreparedRequest,
    operation: WorthServerOperationRequest,
    mutation: WorthServerCompatibilityMutationRequest,
    key: WorthServerIdempotencyKey,
) -> WorthServerCompatibilityMutationOutcome<Execution> {
    worth_proof::compose_transition_outcome(
        super::operation_admission::admit(facade, &prepared, &operation),
        |admission| {
            let diagnostics = prepared.admission().request_context().diagnostics_profile();
            // The no-source adapter cannot honestly enforce these legacy contracts.
            // Refuse them before an effect, rather than silently dropping them.
            for header in [
                "if-match",
                "x-basis-digest",
                "x-base-branch",
                "x-product-session",
                "x-idempotency-binding",
            ] {
                if prepared
                    .request_contract()
                    .canonical_headers()
                    .values(header)
                    .is_some()
                {
                    return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                        WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed,
                        diagnostics,
                        format!("the installed no-source Query adapter does not admit `{header}`"),
                    ));
                }
            }
            if prepared
                .request_contract()
                .normalized_query_pairs()
                .iter()
                .any(|(name, _)| name == "basis" || name == "base-branch")
            {
                return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed, diagnostics,
                "the installed no-source Query adapter does not admit basis or base-branch query preconditions"));
            }
            let Some(binding) = facade.query_handoff.config().application_mutation() else {
                return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                    WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
                    diagnostics,
                    "no Query application mutation binding is installed",
                ));
            };
            let admitted = WorthServerAdmittedQueryMutation::new(
                prepared,
                admission,
                operation.identity().operation_name().to_string(),
                mutation,
                &key,
            );
            TransitionOutcome::Success(Execution::Application(binding.execute(&admitted)))
        },
    )
}
