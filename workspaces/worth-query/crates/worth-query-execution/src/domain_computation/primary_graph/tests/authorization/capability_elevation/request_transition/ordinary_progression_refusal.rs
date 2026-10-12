use super::*;

#[test]
fn ordinary_operation_progression_cannot_authorize_a_lifecycle_request() {
    let world = request_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            request_access(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
                honest_input(),
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = request_operation(&world);

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .authorize_capability_operation(&phase, access, &operation, Default::default())
                .err()
                .expect("the ordinary progression API must reject lifecycle operations")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );
}
