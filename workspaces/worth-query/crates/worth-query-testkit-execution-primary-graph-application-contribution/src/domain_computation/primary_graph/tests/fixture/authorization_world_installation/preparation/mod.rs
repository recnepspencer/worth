use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::*;

pub fn prepare_authorization_world(
    phase: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resources: WorthQueryApplicationQueryResourceProfile,
    relational: Option<worth_relational::facade::runtime::RelationalRuntime>,
    product_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
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
            .prepare_primary_graph_with_relational_runtime_for_test(
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
