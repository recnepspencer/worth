//! The required wave names the demand's stop at its real Product/native selection.
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::rostered_program_revision;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_activation::seed_program_activation;
#[test]
fn required_wave_selection_answers_its_stopped_request_by_value() {
    let mut world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    (*world.application.program_support_mut_for_test()) = Some(installed_program_support(
        world.application.installed_schema(),
    ));
    world
        .application
        .with_host_advancement(|phase| {
            seed_program_activation(
                &phase.borrowed_request_for_test(),
                &world,
                &rostered_program_revision(),
            )
        })
        .unwrap();
    worth_query_execution::test_read_stops::every_consult(|execution| {

            select_required_basis(execution, &world.application, world.application.current_world()).map(|_| ()).map_err(|denial| denial.kind())
        }, WorthQueryOutputDemandDenialKind::Cancelled, WorthQueryOutputDemandDenialKind::SourceQueryExecution(worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None,
        })));
}
