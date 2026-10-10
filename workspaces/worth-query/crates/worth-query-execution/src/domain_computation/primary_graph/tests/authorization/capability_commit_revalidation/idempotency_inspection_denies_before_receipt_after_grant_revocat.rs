//! idempotency inspection denies before receipt after grant revocation.
use super::*;

#[test]
fn idempotency_inspection_denies_before_receipt_after_grant_revocation() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "committed").0
        })
        .expect("the fixture read owns its installed host call");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_operation(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 57);
    world
        .application
        .with_host_advancement(|phase| revoke_grant(&phase, &world))
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            world.application.resolve_admitted_application_idempotency(
                &phase,
                &admission,
                idempotency(57, 57),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("stale capability facts cannot inspect a committed idempotency receipt");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        )
    );
    assert_eq!(
        denial.authorization().map(|denial| denial.kind()),
        Some(WorthQueryOperationAuthorizationDenialKind::StaleAuthorization)
    );
}
