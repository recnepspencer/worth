//! approval after the exact request window returns the request receipt.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn approval_after_the_exact_request_window_returns_the_request_receipt() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 12));
    let request = live_scope();
    let requested = { super::super::request_support::commit_exact_request(&world, &request) };
    super::super::request_support::resolve_exact_request_identities(&world, &request);
    world.authorization_time.script([time(106), time(106)]);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            approval_access(&(phase).test_host_request(), &world, &approver, &request).unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .authorize_elevation_approval(
                    &phase,
                    requested,
                    access,
                    &approval_operation(&world),
                    Default::default(),
                )
                .err()
                .expect("approval after expiry must return the request receipt")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ElevationExpired)
    );
    assert_eq!(
        denial.into_requested().elevation_identity(),
        &string("elevation-2")
    );
}
