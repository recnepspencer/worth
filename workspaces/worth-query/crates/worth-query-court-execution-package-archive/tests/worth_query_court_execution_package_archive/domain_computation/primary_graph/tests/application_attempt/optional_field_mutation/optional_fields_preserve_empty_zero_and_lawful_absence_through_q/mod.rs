//! optional fields preserve empty zero and lawful absence through query.
use crate::domain_computation::primary_graph::tests::application_attempt::optional_field_mutation::*;

#[test]
fn optional_fields_preserve_empty_zero_and_lawful_absence_through_query() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &(phase).test_host_request(),
                &world,
                "unrelated",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                query(
                    &phase,
                    &(phase).borrowed_request_for_test(),
                    &world,
                    &principal,
                    "account-2",
                    &request
                )
                .note(),
                None
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                query(
                    &phase,
                    &(phase).borrowed_request_for_test(),
                    &world,
                    &principal,
                    "account-2",
                    &request
                )
                .score(),
                None
            )
        })
        .expect("the fixture read owns its installed host call");

    let set = world
        .application
        .with_host_advancement(|phase| {
            program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                Some(String::new()),
                Some(0),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            set,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(71, 71),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let present = world
        .application
        .with_host_advancement(|phase| {
            query(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                &principal,
                "account-2",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(present.note(), Some(""));
    assert_eq!(present.score(), Some(0));

    let current = world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &(phase).test_host_request(),
                &world,
                "unrelated",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let clear = world
        .application
        .with_host_advancement(|phase| {
            program(&phase, &world, &principal, &current, &request, None, None)
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            clear,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(72, 72),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let absent = world
        .application
        .with_host_advancement(|phase| {
            query(
                &phase,
                &(phase).borrowed_request_for_test(),
                &world,
                &principal,
                "account-2",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(absent.note(), None);
    assert_eq!(absent.score(), None);
}
