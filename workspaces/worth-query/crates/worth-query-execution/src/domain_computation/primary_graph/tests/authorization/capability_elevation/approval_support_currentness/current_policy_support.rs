use super::*;

#[test]
fn exact_support_revocation_denies_approval_despite_an_equivalent_current_grant() {
    let (world, request, requested) = super::super::approval_transition::requested_world(
        CapabilityElevationScenario::AlternateCurrentGrant,
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            revoke_current_capability(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
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
        panic!("an equivalent grant must not replace the request's exact support lineage");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    assert!(denial.denial().identity().is_some());
    assert_eq!(
        denial.denial().causes(),
        [WorthQueryOperationAuthorizationDenialKind::StaleAuthorization]
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
