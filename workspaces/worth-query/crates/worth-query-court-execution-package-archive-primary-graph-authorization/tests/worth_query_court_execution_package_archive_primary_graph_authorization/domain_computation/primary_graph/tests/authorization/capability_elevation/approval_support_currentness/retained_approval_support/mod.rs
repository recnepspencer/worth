use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_support_currentness::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn replacement_policy_path_for_the_same_grant_cuts_off_approved_use() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    let requester = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(
                &host_custody,
                &world,
                "alice",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::review_support_mutation::replace_support_grantor_with_custodian(
                &host_custody,
                execution,
                &world,
                requester.principal_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    world.authorization_time.script([time(100)]);
    let capability = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::installed_capability(&world);

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&host_custody)
                .admit_approved_elevation_access(
                    &host_custody.phase_for_application(&world.application),
                    &approved,
                    &requester,
                    &capability,
                    crate::domain_computation::primary_graph::tests::authorization::capability_elevation::elevated_input(Some("elevation-2")),
                    &request,
                )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a new allowing path must not replace approved support for active use");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_state::elevation_status(
                    &host_custody,
                    &host_custody.borrowed_request_for_test(),
                    &world
                ),
                CapabilityElevationStatus::Approved
            )
        })
        .expect("the fixture read owns its installed host call");
}
