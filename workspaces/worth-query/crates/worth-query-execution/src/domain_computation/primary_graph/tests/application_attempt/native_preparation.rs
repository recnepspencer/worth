//! Real request interruption at native preparation, before any publication.
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::facade::runtime::ExecutionAllocationPolicy;
use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn native_preparation_cancellation_leaves_head_and_key_available_for_fresh_retry() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(30), source.token());
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let head = world.selected_product().product().selected_commit().clone();
    let program = admitted_program(&world, &principal, &account, &request, "native-prepared");
    // Cancel the actual retained control at the preparation port. Native
    // preparation itself detects it; no transaction error is manufactured.
    world.faults.cancel_next_native_preparation(source);
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(95, 95),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Cancelled),
        "{outcome:?}"
    );
    assert_eq!(&head, world.selected_product().product().selected_commit());
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    let fresh = live_scope();
    let unchanged = resolved_account(&world, "open", &fresh);
    assert_eq!(unchanged.entity_id(), account.entity_id());
    let principal = authenticated_principal(&world, &fresh);
    let retry = admitted_program(&world, &principal, &unchanged, &fresh, "native-prepared");
    let outcome = world.application.compare_and_commit_application(
        retry,
        idempotency(95, 95),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the cancelled preparation must not consume its idempotency key: {outcome:?}"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        1
    );
    let landed = resolved_account(&world, "native-prepared", &live_scope());
    assert_eq!(landed.entity_id(), account.entity_id());
}
