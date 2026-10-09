use super::*;

pub(super) fn prepare_authorization_world(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resources: WorthQueryApplicationQueryResourceProfile,
    relational: Option<worth_relational::facade::runtime::RelationalRuntime>,
    product_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
) -> PreparedAuthorizationWorld {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(portable_package(declaration.clone()))
        .unwrap();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .application_query_resources(resources)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap();
    let (runtime, authority) = installation.into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    let bootstrap = match relational {
        Some(relational) => authority
            .prepare_primary_graph_with_relational_runtime(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                relational,
                product_resources,
            )
            .unwrap(),
        None => authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                product_resources,
            )
            .unwrap(),
    };
    PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        bootstrap,
    }
}
