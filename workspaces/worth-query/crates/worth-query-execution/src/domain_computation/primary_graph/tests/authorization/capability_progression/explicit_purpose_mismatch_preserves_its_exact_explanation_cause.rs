//! explicit purpose mismatch preserves its exact explanation cause.
use super::*;

#[test]
fn explicit_purpose_mismatch_preserves_its_exact_explanation_cause() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
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
    let mut input = capability_input(100);
    input.purpose = CapabilityPurpose::Audit;

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_capability_access(&phase, &principal, &capability, input, &request)
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an explicit purpose mismatch must not mint capability authority")
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::PurposeMismatch
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::PurposeMismatch)
    );
}
