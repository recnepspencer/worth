//! preparation rejection is denied without effect or idempotency residue.
use crate::domain_computation::primary_graph::tests::application_attempt::terminal_failures::*;

#[test]
fn preparation_rejection_is_denied_without_effect_or_idempotency_residue() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "prepared-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.reject_next_session_prepare();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(19, 19),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("provider preparation rejection must be a typed denial");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderPlan
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");

    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "prepared-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(19, 19),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
