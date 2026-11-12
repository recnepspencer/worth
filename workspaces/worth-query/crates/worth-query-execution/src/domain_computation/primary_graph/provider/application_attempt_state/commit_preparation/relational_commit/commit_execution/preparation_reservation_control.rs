//! Private early-admission probe over a genuine production-produced Query.
//! Memory is used only to exercise the owner-local pre-World refusal; this is
//! not evidence of Directory publication or durable refusal production.
use super::durable_custody::QueryPreparingApplicationCommit;
use super::*;
use crate::domain_computation::primary_graph::provider::{
    WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution,
};

pub(super) fn refuse_duplicate_before_world_for_test(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: QueryPreparingApplicationCommit,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    assert!(query.is_unprepared());
    let product = query.attempt.affinity().product_publication().clone();
    let binding = query.attempt.idempotency();
    let affinity = WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
    let winner = provider
        .reserve_query_before_world(&product, binding)
        .expect("real winning subject slot");
    let outcome = durable::commit(execution, provider, query, allocation);
    assert!(
        matches!(&outcome, Err(crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductPreparationRefused(refused))
        if matches!(refused.reason(), crate::domain_computation::WorthQueryProductPreparationRefusal::QueryPreparationReservationRefused))
    );
    assert!(matches!(
        provider.resolve_unpublished_application_idempotency(&affinity, binding),
        Some(WorthQueryProviderIdempotencyResolution::PreparationBusy)
    ));
    let mut different_intent = *binding.intent_identity();
    different_intent[0] ^= 1;
    let drift =
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::new(
            *binding.key_identity(),
            different_intent,
        );
    assert!(
        matches!(
            provider.resolve_unpublished_application_idempotency(&affinity, drift),
            Some(WorthQueryProviderIdempotencyResolution::Drift)
        ),
        "intent drift precedes unfilled-slot Busy"
    );
    // The actual original guard removes only its exact row. The losing attempt
    // has neither removed nor replaced it. Dispose the winner exactly once.
    drop(winner);
    assert!(provider
        .resolve_unpublished_application_idempotency(&affinity, binding)
        .is_none());
    outcome
}
