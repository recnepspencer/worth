use super::*;

pub(super) fn publish_authorization_world(
    prepared: PreparedAuthorizationWorld,
) -> AuthorizationWorld {
    let PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        mut bootstrap,
    } = prepared;
    let program_required = schema
        .installed_mutation_binding::<ProgramRequiredMutationBinding>()
        .unwrap();
    bootstrap
        .install_handler(&program_required, ProgramRequiredHandler)
        .unwrap();
    let capability_touch = schema
        .installed_mutation_binding::<CapabilityTouchMutationBinding>()
        .unwrap();
    bootstrap
        .install_handler(&capability_touch, CapabilityTouchHandler)
        .unwrap();

    let invariant = bootstrap.retain_invariant_projection_authority();
    let authorization_time = AuthorizationTimeController::default();
    let faults = std::sync::Arc::new(
        crate::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController::default(),
    );
    let application = bootstrap
        .publish_application_runtime_with_ports(
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
