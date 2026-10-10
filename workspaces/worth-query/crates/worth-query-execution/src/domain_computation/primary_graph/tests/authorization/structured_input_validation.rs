use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{
    installed_capability_authorization_world, live_scope, CapabilityTouchOperation,
    TouchAccountCapability,
};
use super::capability_progression::{capability_input, time};
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn structured_input_validation_is_enforced_by_capability_admission() {
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
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let input = capability_input(u64::MAX);

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_capability_access(&phase, &principal, &capability, input, &request)
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an invalid structured input must not mint capability authority")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::InvalidOperationInput
    );
}
