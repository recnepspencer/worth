use super::{
    authenticated, idempotency, link_program, live_scope, resolved_account, resolved_principal,
    WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_two_principal_authorization_world;

#[test]
fn unrelated_adjacency_growth_preserves_prepared_decision_facts() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_two_principal_authorization_world(false);
    let request = live_scope();
    let alice = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "alice",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                "bob",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let alice_identity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                1,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob_identity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_principal(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                2,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first_account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let second_account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();
    let alice_program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            link_program(
                &phase,
                &world,
                &alice,
                &alice_identity,
                &first_account,
                &request,
                "open",
                "alice-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    let bob_program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            link_program(
                &phase,
                &world,
                &bob,
                &bob_identity,
                &second_account,
                &request,
                "unrelated",
                "bob-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            bob_program,
            idempotency(33, 33),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let current = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_ne!(current.selected_commit(), selected.selected_commit());
    assert_eq!(commit_count(), baseline + 1);

    let outcome = world.application.compare_and_commit_application(
        alice_program,
        idempotency(34, 34),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "unchanged facts must commit on the newly admitted product basis: {outcome:?}"
    );
    assert_eq!(commit_count(), baseline + 2);
    let successor = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_ne!(successor.selected_commit(), current.selected_commit());

    let readmitted = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            link_program(
                &phase,
                &world,
                &alice,
                &alice_identity,
                &first_account,
                &request,
                "open",
                "alice-owner",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            readmitted,
            idempotency(34, 34),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
}
