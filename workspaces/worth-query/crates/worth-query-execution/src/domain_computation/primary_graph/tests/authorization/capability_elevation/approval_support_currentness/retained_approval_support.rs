use super::*;

#[test]
fn replacement_policy_path_for_the_same_grant_cuts_off_approved_use() {
    let (world, request, approved) = super::super::approval_transition::exact_approved_world();
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
    world.authorization_time.script([time(100)]);
    let capability = super::super::installed_capability(&world);

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
                    super::super::elevated_input(Some("elevation-2")),
                    &request,
                )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a new allowing path must not replace approved support for active use");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                super::super::terminal_state::elevation_status(
                    &host_custody,
                    &host_custody.borrowed_request_for_test(),
                    &world
                ),
                CapabilityElevationStatus::Approved
            )
        })
        .expect("the fixture read owns its installed host call");
}
