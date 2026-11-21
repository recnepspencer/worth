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
pub(in crate::domain_computation::primary_graph::conditional_operation) fn continue_recovered_original<
    Schema,
>(
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
            Ok(Continuation::Published { commit }) => Outcome::AlreadyCommitted(
                commit.owner_identity().durable().map(|owner| {
                    worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                        owner,
                        commit.ordinal(),
                    )
                }),
            ),
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

/// Consult World's owning fate for a live indeterminate decision when the
/// current candidate disappeared or changed revision. Never finishes the
/// obligation unless World itself establishes publication, release, or proven
/// absence with no live pending/unpublished original.
pub(super) fn resolve_live_indeterminate_world_fate<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    key: [u8; 32],
    intent: [u8; 32],
) -> Outcome {
    use crate::domain_computation::primary_graph::provider::WorthQueryConditionalKeyFate as Fate;
    let provider = &runtime.runtime.primary_provider;
    let publication = product.publication_binding();
    match provider.conditional_key_fate(execution, &publication, key, intent) {
        Ok(Fate::LiveUnpublishedOrPending) => live_unpublished_unresolved(),
        Ok(Fate::Published(receipt)) => Outcome::AlreadyCommitted(
            receipt
                .new_product_head()
                .durable()
                .map(|head| head.selected_commit()),
        ),
        Ok(Fate::Released) => Outcome::AlreadyCommitted(None),
        Ok(Fate::Absent) => Outcome::RetiredNoEffect,
        Ok(Fate::RecoveredUnpublished(_)) => {
            let joined = runtime
                .installed_program_support()
                .and_then(|support| {
                    match support
                        .lifecycle()
                        .retained_indeterminate_original(&key)
                    {
                        crate::domain_computation::primary_graph::query_obligations::QueryConditionalOriginal::Recovered(original) => Some(original),
                        crate::domain_computation::primary_graph::query_obligations::QueryConditionalOriginal::Unclassified => None,
                    }
                });
            continue_recovered_original(
                execution,
                runtime,
                product,
                WorthQueryApplicationIdempotencyBinding::new(key, intent),
                joined,
            )
            .unwrap_or_else(|| unresolved("recovered original disappeared before continuation"))
        }
        Ok(Fate::OtherIntent) | Err(_) => {
            unresolved("World did not resolve the indeterminate key")
        }
    }
}

pub(super) fn live_unpublished_unresolved() -> Outcome {
    unresolved("live unpublished or pending original still held for indeterminate key")
}

/// The decision stays indeterminate; its row keeps the original association.
fn unresolved(detail: &'static str) -> Outcome {
    Outcome::Indeterminate(Failure::UnresolvedCommit(unknown_commit_recovery_evidence(
        detail,
    )))
}
