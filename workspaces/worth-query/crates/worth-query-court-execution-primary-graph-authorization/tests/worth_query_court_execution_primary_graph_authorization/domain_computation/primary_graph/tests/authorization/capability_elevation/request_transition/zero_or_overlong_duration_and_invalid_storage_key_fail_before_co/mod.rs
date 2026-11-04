//! zero or overlong duration and invalid storage key fail before commit.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn zero_or_overlong_duration_and_invalid_storage_key_fail_before_commit() {
    for duration in [Duration::ZERO, Duration::from_secs(1_201)] {
        let world = request_world();
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let access = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                request_access(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &principal,
                    &request,
                    RequestElevationInput {
                        duration,
                        ..honest_input()
                    },
                )
                .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .authorize_elevation_request(
                        &host_custody.phase_for_application_for_test(&world.application),
                        access,
                        &request_operation(&world),
                        Default::default(),
                    )
                    .err()
                    .expect("invalid duration must not mint request authority")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationDurationExceeded
            )
        );
    }

    let world = request_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            request_access(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &principal,
                &request,
                RequestElevationInput {
                    elevation_key: " bad-key".to_owned(),
                    ..honest_input()
                },
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .authorize_elevation_request(
                    &host_custody.phase_for_application_for_test(&world.application),
                    access,
                    &request_operation(&world),
                    Default::default(),
                )
                .err()
                .expect("invalid provider key must fail before lifecycle graph progression")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationRequestRejected
        )
    );
}
