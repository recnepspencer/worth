//! revoked access cannot progress to operation authority.
use super::*;

#[test]
fn revoked_access_cannot_progress_to_operation_authority() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_access(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
                100,
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            revoke_grant(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.application.authorize_capability_operation(
                &phase,
                access,
                &operation,
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("revoked retained access cannot become operation authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}
