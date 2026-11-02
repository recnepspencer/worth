//! response loss resolves the published commit before returning.
use super::*;

#[test]
fn response_loss_resolves_the_published_commit_before_returning() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_expected_status(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                ("open", "published"),
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_expected_status(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                ("open", "published"),
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) =
        world.application.compare_and_commit_application(
            first,
            idempotency(15, 15),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("authoritative idempotency must prove the response-lost commit");
    };
    let WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(15, 15),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("retry must recover the transaction published before response loss");
    };
    crate::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(
        &receipt,
        &first_receipt,
    );
    assert!(receipt.new_product_head().memory().is_some());
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
    assert!(receipt.changed_record_count() >= 2);
}
