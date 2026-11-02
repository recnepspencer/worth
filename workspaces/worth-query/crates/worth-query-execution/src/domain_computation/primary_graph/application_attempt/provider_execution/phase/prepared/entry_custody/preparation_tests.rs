use super::*;
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{
        admitted_program, authenticated_principal, idempotency, resolved_account,
    },
    fixture::*,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
fn requested_program(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
> {
    world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), world, request);
            let account = resolved_account(&phase.test_host_request(), world, "open", request);
            admitted_program(&phase, world, &principal, &account, request, "closed")
        })
        .unwrap()
}
#[test]
fn entry_custody_preparation_refreshes_lapse_before_program_fact_work() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let mut program = requested_program(&world, &request);
    // Inject another real result's routes: the facts cannot satisfy this binding.
    let donor = requested_program(&world, &request);
    program.read_set.effect_routes = donor.read_set.effect_routes;
    assert!(!program
        .read_set
        .effect_routes
        .matches_body(&program.read_set.facts));
    world
        .application
        .with_host_advancement(|phase| {
            let entry = world
                .application
                .validate_commit_program_in_advancement(&phase, program)
                .unwrap_or_else(|outcome| panic!("fresh: {outcome:?}"));
            cancellation.cancel();
            let outcome = prepare_application_commit(
                &phase.borrowed_request_for_test(),
                WorthQueryApplicationCommitPreparationRequest::new(
                    entry,
                    idempotency(215, 216),
                    None,
                    None,
                ),
            );
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitPreparation::Terminal(
                    WorthQueryApplicationCommitOutcome::Cancelled
                )
            ));
        })
        .unwrap();
}
