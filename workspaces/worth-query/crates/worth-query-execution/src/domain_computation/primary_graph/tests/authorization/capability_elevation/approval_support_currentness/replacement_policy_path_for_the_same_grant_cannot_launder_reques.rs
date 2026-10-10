//! replacement policy path for the same grant cannot launder request support.
use super::*;

#[test]
fn replacement_policy_path_for_the_same_grant_cannot_launder_request_support() {
    let (world, request, requested) =
        super::super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let requester = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::approval_transition::authenticated(
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
            super::super::mutation::replace_support_grantor_with_custodian(
                &host_custody,
                execution,
                &world,
                requester.principal_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::approval_transition::authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::approval_transition::approval_access(
                &host_custody,
                &world,
                &approver,
                &request,
            )
            .expect("the independent approval command grant remains current")
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.application.authorize_elevation_approval(
                &host_custody.phase_for_application(&world.application),
                requested,
                access,
                &super::super::approval_transition::approval_operation(&world),
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a new allowing path must not replace the request receipt's original path");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_requested_without_approval(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                &denial.into_requested(),
                approver.principal_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
}
