//! The transaction's application and branch custody precede admission revalidation.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedChange, WorthQueryProductTransactionCommitError,
};
use std::time::SystemTime;

fn candidate(
    world: &super::super::fixture::AuthorizationWorld,
    lapsed: bool,
) -> program_fixture::Program {
    world.authorization_time.hold(SystemTime::now());
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .unwrap();
    let mut program = world
        .application
        .with_host_advancement(|phase| {
            let account = resolved_account(&phase.test_host_request(), world, "open", &request);
            admitted_program(&phase, world, &principal, &account, &request, "closed")
        })
        .unwrap();
    program
        .validate_current_authority()
        .expect("the genuine candidate starts fresh");
    if lapsed {
        program.expire_authentication_for_test();
        assert_eq!(
            program.validate_current_authority().unwrap_err().kind(),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication,
            ),
            "the admission's authentication really expired"
        );
    } else {
        program
            .validate_current_authority()
            .expect("the control remains fresh");
    }
    program
}

#[test]
fn transaction_application_mismatch_precedes_fresh_and_lapsed_admission_checks() {
    let source = installed_authorization_world(true);
    let target = installed_authorization_world(true);
    for lapsed in [false, true] {
        let program = candidate(&source, lapsed);
        let outcome = target
            .application
            .on_branch(target.application.current_world())
            .transaction()
            .apply(WorthQueryAdmittedChange::new(program, idempotency(41, 41)))
            .commit();
        assert!(
            matches!(
                outcome,
                Err(WorthQueryProductTransactionCommitError::ApplicationMismatch)
            ),
            "foreign application custody is the documented error: {outcome:?}"
        );
    }
}

#[test]
fn transaction_branch_mismatch_precedes_fresh_and_lapsed_admission_checks() {
    let world = installed_authorization_world(true);
    let sibling = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    for lapsed in [false, true] {
        let program = candidate(&world, lapsed);
        assert_ne!(program.product_branch(), sibling);
        let outcome = world
            .application
            .on_branch(sibling)
            .transaction()
            .apply(WorthQueryAdmittedChange::new(program, idempotency(42, 42)))
            .commit();
        assert!(
            matches!(
                outcome,
                Err(WorthQueryProductTransactionCommitError::BranchMismatch)
            ),
            "foreign branch custody is the documented error: {outcome:?}"
        );
    }
}
