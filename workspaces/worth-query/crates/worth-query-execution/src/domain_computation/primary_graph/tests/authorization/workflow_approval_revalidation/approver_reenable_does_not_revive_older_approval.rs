//! approver reenable does not revive older approval.
use super::*;

#[test]
fn approver_reenable_does_not_revive_older_approval() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let basis = durable_approval_basis(&admission);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            disable_mapping(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.mapping_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            enable_mapping(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.mapping_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .readmit_workflow_approval_authority(
                    &host_custody.borrowed_request_for_test(),
                    &basis,
                    &admission,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal
    );
}
