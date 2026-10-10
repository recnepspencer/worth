//! response loss resolves the published commit before returning.
use super::*;

#[test]
fn response_loss_resolves_the_published_commit_before_returning() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(15, 15),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("retry must recover the transaction published before response loss");
    };
    assert!(receipt.is_same_authoritative_commit(&first_receipt));
    assert_eq!(receipt.outcome_identity(), first_receipt.outcome_identity());
    assert_eq!(
        first_receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        first_receipt.terminal().attempt_resources_released(),
        Some(true)
    );
    assert_eq!(
        receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(
        receipt.precondition_comparison().expected_version_count(),
        0
    );
    assert_eq!(receipt.precondition_comparison().expected_fact_count(), 1);
    assert!(receipt.changed_record_count() >= 2);
}
