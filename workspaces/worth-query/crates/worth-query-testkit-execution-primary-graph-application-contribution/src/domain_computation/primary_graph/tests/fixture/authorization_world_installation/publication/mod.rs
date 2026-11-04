use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::*;

pub fn publish_authorization_world(
    phase: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    prepared: PreparedAuthorizationWorld,
    admission_clock: worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
) -> AuthorizationWorld {
    let PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        mut bootstrap,
    } = prepared;
    crate::domain_computation::primary_graph::tests::fixture::handler_installation::install_fixture_handlers(&schema, &mut bootstrap);

    let invariant = bootstrap.retain_invariant_projection_authority();
    let authorization_time = AuthorizationTimeController::on_clock(admission_clock);
    let faults = std::sync::Arc::new(
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController::default(),
    );
    let application = bootstrap
        .publish_application_runtime_with_ports_for_test(
            &phase.bootstrap_for_test(),
            runtime,
            authority,
            schema,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            authorization_time.clone(),
            faults.clone(),
        )
        .unwrap();
    AuthorizationWorld {
        application,
        binding,
        invariant,
        authorization_time,
        faults,
    }
}
