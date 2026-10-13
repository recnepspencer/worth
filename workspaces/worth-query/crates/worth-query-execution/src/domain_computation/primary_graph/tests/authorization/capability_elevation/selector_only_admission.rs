//! An elevation selector cannot replace the approval transition.
use super::*;
#[test]
fn selector_only_admission_cannot_bypass_the_approval_transition() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            admit_raw(&phase, &world, &principal, &request, Some("elevation-1"))
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a raw selector must not substitute for approved lifecycle authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationTransitionRequired
        )
    );
}
