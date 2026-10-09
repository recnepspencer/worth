use super::*;

pub(super) fn publish_authorization_world(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    prepared: PreparedAuthorizationWorld,
) -> AuthorizationWorld {
    let PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        mut bootstrap,
    } = prepared;
    super::super::handler_installation::install_fixture_handlers(&schema, &mut bootstrap);

    let invariant = bootstrap.retain_invariant_projection_authority();
    let authorization_time = AuthorizationTimeController::default();
    let faults = std::sync::Arc::new(
        crate::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController::default(),
    );
    let application = bootstrap
        .publish_application_runtime_with_ports(
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
