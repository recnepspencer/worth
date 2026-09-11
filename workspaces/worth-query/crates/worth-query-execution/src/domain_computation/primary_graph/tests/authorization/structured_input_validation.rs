use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{
    installed_capability_authorization_world, live_scope, CapabilityGovernedInputIdentity,
    CapabilityTouchOperation, TouchAccountCapability,
};
use super::capability_progression::{capability_input, time};
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn structured_input_validation_is_enforced_by_capability_admission() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let input = capability_input(u64::MAX, CapabilityGovernedInputIdentity::None);

    let Err(denial) =
        world
            .selected_product()
            .admit_capability_access(&principal, &capability, input, &request)
    else {
        panic!("an invalid structured input must not mint capability authority")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::InvalidOperationInput
    );
}
