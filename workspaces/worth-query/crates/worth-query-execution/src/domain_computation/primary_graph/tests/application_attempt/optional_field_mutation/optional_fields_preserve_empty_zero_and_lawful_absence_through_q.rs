//! optional fields preserve empty zero and lawful absence through query.
use super::*;

#[test]
fn optional_fields_preserve_empty_zero_and_lawful_absence_through_query() {
    let world = super::super::super::fixture::installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::resolved_account(
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            super::super::idempotency(71, 71),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let present = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            super::super::resolved_account(
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
            let host_custody = phase.test_host_request();
            program(&phase, &world, &principal, &current, &request, None, None)
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            clear,
            super::super::idempotency(72, 72),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let absent = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
