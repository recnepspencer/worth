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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let head = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "native-prepared",
            )
        })
        .expect("the fixture read owns its installed host call");
    // Cancel the actual retained control at the preparation port; the next
    // request-paid index charge detects it before publication.
    world.faults.cancel_next_native_preparation(source);
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(95, 95),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the request-paid seam retains its typed interruption: {outcome:?}");
    };
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                &head,
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit()
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    let fresh = live_scope();
    let unchanged = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &fresh)
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(unchanged.entity_id(), account.entity_id());
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &fresh)
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &unchanged,
                &fresh,
                "native-prepared",
            )
        })
        .expect("the fixture read owns its installed host call");
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
    let landed = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request(),
                &world,
                "native-prepared",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(landed.entity_id(), account.entity_id());
}
