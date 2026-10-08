use super::*;

#[test]
fn ordinary_operation_progression_cannot_authorize_a_lifecycle_request() {
    let world = request_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = request_access(&world, &principal, &request, honest_input()).unwrap();
    let operation = request_operation(&world);

    let denial = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .err()
        .expect("the ordinary progression API must reject lifecycle operations");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );
}
