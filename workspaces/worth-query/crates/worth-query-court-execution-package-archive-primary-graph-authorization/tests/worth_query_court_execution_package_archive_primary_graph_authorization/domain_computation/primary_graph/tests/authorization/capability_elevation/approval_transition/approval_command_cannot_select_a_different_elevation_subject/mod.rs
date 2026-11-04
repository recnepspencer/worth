//! approval command cannot select a different elevation subject.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn approval_command_cannot_select_a_different_elevation_subject() {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            approval_access_for_elevation(&host_custody, &world, &approver, &request, "elevation-1")
                .expect("the approver independently holds command authority for elevation-1")
        })
        .expect("the fixture read owns its installed host call");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .authorize_elevation_approval(
                    &host_custody.phase_for_application(&world.application),
                    requested,
                    access,
                    &approval_operation(&world),
                    Default::default(),
                )
                .err()
                .expect("command authority for another subject must not consume this request")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApprovalRejected
        )
    );
}
