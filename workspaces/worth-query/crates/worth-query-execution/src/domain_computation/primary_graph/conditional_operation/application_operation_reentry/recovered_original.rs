//! A retry of a conditional key whose original was recovered unpublished from
//! an ended incarnation continues that exact original through World's
//! recovery port. It never invokes the operation again and never publishes a
//! second original. A row the cold consumer joined accepts only its own.
use super::WorthQueryTemporalReentryOutcome as Outcome;
use crate::domain_computation::primary_graph::application_attempt::{
    business_original, provider_execution::recovery_evidence::unknown_commit_recovery_evidence,
};
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalReentryFailure as Failure;
use crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome as Continuation;

/// `None` when the key has no recovered original: the ordinary commit path
/// then resolves the key itself before any effect.
pub(super) fn continue_recovered_original<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    binding: WorthQueryApplicationIdempotencyBinding,
    joined: Option<DurableCompositePublicationAttemptIdentity>,
) -> Option<Outcome> {
    let provider = &runtime.runtime.primary_provider;
    let publication = product.publication_binding();
    let handle = provider.recovered_conditional_original(publication.observation(), binding)?;
    // Same-key retries serialize on the branch commit lane, as commits do.
    let lane = provider.application_branch_commit_lane(publication.observation());
    let _coordination = lane.enter();
    let effects = match provider.recovered_unpublished_application(&publication, handle) {
        Ok(effects) => effects,
        Err(_) => {
            return Some(unresolved(
                "World refused to inspect the recovered original",
            ))
        }
    };
    let actual = business_original::from_attempt(effects.attempt_identity());
    if joined.is_some_and(|joined| actual != Some(QueryManagedCleanupOriginal::Attempt(joined))) {
        return Some(unresolved(
            "the key's recovered original is not the joined original",
        ));
    }
    Some(
        match effects
            .into_recovery()
            .continue_original_in_request(execution)
        {
            Ok(Continuation::Published { .. }) => Outcome::AlreadyCommitted,
            Ok(
                Continuation::Pending(_)
                | Continuation::Refused { .. }
                | Continuation::EvidenceRequired(_)
                | Continuation::Indeterminate(_),
            )
            | Err(_) => unresolved("the recovered original did not publish; a retry continues it"),
        },
    )
}

/// The decision stays indeterminate; its row keeps the original association.
fn unresolved(detail: &'static str) -> Outcome {
    Outcome::Indeterminate(Failure::UnresolvedCommit(unknown_commit_recovery_evidence(
        detail,
    )))
}
