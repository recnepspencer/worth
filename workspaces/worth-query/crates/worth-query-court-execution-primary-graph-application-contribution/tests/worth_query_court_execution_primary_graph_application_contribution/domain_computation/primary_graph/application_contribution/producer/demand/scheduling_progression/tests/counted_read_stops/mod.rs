use crate::domain_computation::primary_graph::application_contribution::producer::demand::scheduling_progression::tests::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::observed_source_fixture::observed_account_source;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Kind;

#[test]
fn scheduling_progression_preserves_admission_stops_at_every_consult() {
    let mut world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    (*world.application.program_support_mut_for_test()) = Some(
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support(
            world.application.installed_schema(),
        ),
    );
    world
        .application
        .with_host_advancement(|phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_activation::seed_program_activation(
            &phase.borrowed_request_for_test(), &world,
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::rostered_program_revision(),
        );
        })
        .unwrap();
    let source = observed_account_source(&world);
    let (_, route) = installation();
    (*world.application.output_production_mut_for_test().producer_routes_mut_for_test()).insert(
        "scheduling-busy".to_owned(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::scheduling::WorthQueryInstalledOutputProducerRoute::new_for_test(route, None),
    );
    let selected = WorthQuerySelectedApplicationProducer::new_for_test("scheduling-busy".to_owned(), WorthQueryProducerApplicability::new(
            "account",
            worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture::Initial,
        ), false, None, None, false);
    // The installed route is foreign to this fixture's bridge. Its ordinary
    // refusal is the baseline; the actual scheduling admission read must still
    // propagate every stop before the bridge is contacted.
    world.application.with_host_advancement(|phase| {
        worth_query_execution::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(),
            |execution| world.application.schedule_selected_output_producer_for_test(
                execution, &phase, &selected, world.application.current_world(), &source, None,
             ).map(|_| ()).map_err(|denial| denial.kind()).or_else(|kind| {
                if matches!(&kind, WorthQueryOutputDemandDenialKind::BridgeConditional(native)
                    if matches!(native.as_ref(), worth_runtime_bridge::facade::BridgeConditionalDenialKind::StaleLowering)) {
                    Ok(())
                } else {
                    Err(kind)
                }
            }),
            WorthQueryOutputDemandDenialKind::Cancelled,
            WorthQueryOutputDemandDenialKind::SourceQueryExecution(
                worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(Kind::ExecutionResource {
                    denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None, policy_ancestor: None,
                }),
            ),
        );
    }).unwrap();
}
