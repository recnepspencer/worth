//! support expiry denies approval while the request lifecycle is current.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_support_currentness::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn support_expiry_denies_approval_while_the_request_lifecycle_is_current() {
    let (world, request, requested) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::requested_world(
        CapabilityElevationScenario::ExpiringSupport,
    );
    world
        .authorization_time
        .script(std::iter::repeat_n(time(103), 8));
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_access(
                &host_custody,
                &world,
                &approver,
                &request,
            )
            .expect("the approval command and request lifecycle remain current at time 103")
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world.application.authorize_elevation_approval(
                &host_custody.phase_for_application(&world.application),
                requested,
                access,
                &crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_operation(&world),
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("the exact support expired at time 102 and must be reobserved");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )
    );
    assert!(denial.denial().identity().is_some());
    assert_eq!(
        denial.denial().causes(),
        [WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )]
    );
    let requested = denial.into_requested();
    assert_eq!(
        requested.expires_at(),
        &worth_foundational::facade::AspectValue::UInt64(105)
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert_requested_without_approval(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                &requested,
                approver.principal_entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");
}
