//! parent reactivation does not revive delegated approval.
use super::*;

#[test]
fn parent_reactivation_does_not_revive_delegated_approval() {
    let world = installed_delegated_capability_world();
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
            revoke_parent(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            update_grant_field(
                &phase,
                &world,
                "capability-parent",
                field(
                    &world,
                    super::super::super::fixture::capability::CapabilityStatusField::reference(),
                ),
                super::super::super::fixture::CapabilityStatusBinding::encode(
                    &super::super::super::fixture::CapabilityStatus::Active,
                )
                .unwrap(),
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
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}
