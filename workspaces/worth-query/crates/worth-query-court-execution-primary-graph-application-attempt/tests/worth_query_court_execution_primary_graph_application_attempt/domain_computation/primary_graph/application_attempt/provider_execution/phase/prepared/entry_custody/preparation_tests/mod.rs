use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub fn requested_program(
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
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
