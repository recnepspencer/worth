use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedCapabilityTouchOperation;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::admit;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAuthorizationExplanationCause;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn explicitly_expired_elevation_has_a_typed_denial() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::mutation::set_status(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                CapabilityElevationStatus::Expired,
            )
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("explicit expired posture must not mint active-use authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ElevationExpired)
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ElevationExpired)
    );
}

#[test]
fn trusted_time_denies_active_status_past_the_elevation_expiry() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(106)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("trusted time must terminate active-use authority at elevation expiry");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ElevationExpired)
    );
}

#[test]
fn active_status_before_elevation_issue_is_inactive_not_expired() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(99)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an elevation cannot open authority before its installed issue boundary");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationInactive
        )
    );
}

#[test]
fn expiry_between_access_and_operation_denies_fresh_progression() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(106)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world.application.authorize_capability_operation(
                &host_custody.phase_for_application(&world.application),
                access,
                &operation,
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an access proof must not progress after the exact elevation expires");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ElevationExpired)
    );
}
