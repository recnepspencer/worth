//! caller time cannot substitute for the query owned sample.
use super::*;

#[test]
fn caller_time_cannot_substitute_for_the_query_owned_sample() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(300)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
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
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a descriptive in-range caller time cannot revive an expired grant");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing]
    );
}
