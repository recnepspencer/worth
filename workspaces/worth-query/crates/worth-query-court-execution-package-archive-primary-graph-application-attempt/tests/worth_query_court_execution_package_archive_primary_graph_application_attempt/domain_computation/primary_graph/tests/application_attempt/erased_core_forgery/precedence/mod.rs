//! Precedence: a forged core that has also lapsed is refused as forged,
//! in the slot ownership holds today, at every typed entry.

use crate::process::domain_computation::primary_graph::tests::application_attempt::erased_core_forgery::*;

/// Mints `count` admissions under one short-lived authentication.
pub fn expiring(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    count: usize,
) -> (Vec<Admission>, SystemTime, u64) {
    let external = world.authenticate("alice", Duration::from_secs(2), request);
    let expires_at = external.expires_at();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, account) = party(world, request);
    let admissions = (0..count)
        .map(|_| {
            admit!(
                world,
                &principal,
                &account,
                ProgramRequiredOperation,
                request
            )
        })
        .collect();
    (admissions, expires_at, *principal.principal_identity())
}

#[test]
fn a_forged_and_expired_core_is_refused_as_forged_at_every_entry() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (mut admissions, expires_at, identity) = expiring(&world, &request, 3);
    for admission in &mut admissions {
        admission.flip_core_seal_byte_for_test();
    }
    let until_expiry = expires_at
        .duration_since(SystemTime::now())
        .unwrap_or_default();
    std::thread::sleep(until_expiry + Duration::from_millis(10));
    assert_eq!(
        admissions[0]
            .validate_current_authority()
            .unwrap_err()
            .kind(),
        refused(AuthorizationRefusal::ExpiredAuthentication),
        "the forged core has lapsed on its own"
    );

    let projection = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admissions[0],
                    |_, _| (),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("a forged core never projects")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(refused(
            AuthorizationRefusal::StaleInstalledOperation
        ))
    );

    let resolution = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admissions[0], idempotency(9, 9))
                .expect_err("a forged core never resolves idempotency")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        resolution.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );

    let read = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            world
                .application
                .begin_application_read_attempt(
                    &custody.borrowed_request_for_test(),
                    admissions.remove(0),
                )
                .err()
                .expect("a forged core never begins a read attempt")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        read.kind(),
        WorthQueryApplicationAttemptDenialKind::ForeignApplication
    );

    let execution = execute(&world, identity, admissions.remove(0));
    assert!(execution.did_nothing());
    assert_projection_refused(
        execution.denial,
        AuthorizationRefusal::StaleInstalledOperation,
    );
}
