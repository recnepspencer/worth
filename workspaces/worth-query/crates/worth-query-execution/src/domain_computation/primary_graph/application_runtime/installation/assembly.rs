use super::PublishedApplicationGraph;
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::primary_graph::authentication_clock::WorthQueryAuthenticationClock;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
pub(super) fn assemble_application_runtime<Schema>(
    graph: PublishedApplicationGraph<
        crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge,
    >,
    installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    conditional_operations:
        crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>,
    mutation_handlers: crate::domain_computation::primary_graph::handler::InstalledMutationHandlerRegistry<Schema>,
    mutation_projection: crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    output_producer_routes:
        crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputProducerRoutes,
    output_readiness_routes:
        crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let product_runtime = crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime::install(
        graph.primary_provider.graph.prepare_product_source(&graph.relational_branch_identity)
            .map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
                format!("Product source admission: {denial:?}"),
            ))?,
        &mut graph.bridge.conditional_lifecycle(),
        graph.product_world_resources,
    ).map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        denial.detail(),
    ))?;
    let runtime_authority = graph.runtime.authority_identity();
    let schema_binding = installed_schema.binding_identity();
    let application_readiness_schema_token = format!(
        "{}:{}:{}",
        schema_binding.generation(),
        schema_binding.package_identity().render_hex(),
        schema_binding.schema_identity().render_hex(),
    );
    let granular_invalidation = crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation::new(
        schema_binding.clone(),
        graph.primary_provider.graph.clone(),
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot::new(
            product_runtime.clone(),
            graph.bridge.conditional_operations(),
        ),
    );
    // One clock, shared. The registry hands it back to any handle that needs to
    // re-check its own deadline, which is why no recovery transition takes a
    // clock argument (R8.31).
    let authorization_clock = std::sync::Arc::new(authorization_clock);
    let recovery_handles = std::sync::Arc::new(
        crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry::for_runtime(
            runtime_authority,
            std::sync::Arc::clone(&authorization_clock),
        ),
    );
    Ok(WorthQueryPrimaryGraphApplicationRuntime {
        runtime: graph.runtime,
        installed_schema,
        application_readiness_schema_token,
        publication: graph.publication,
        authorization,
        authorization_clock,
        authentication_clock: WorthQueryAuthenticationClock::system(),
        relational_branch_identity: graph.relational_branch_identity,
        bridge: graph.bridge,
        product_runtime,
        granular_invalidation,
        conditional_operations: std::sync::Mutex::new(conditional_operations),
        primary_provider: graph.primary_provider,
        primary_graph_authority: graph.primary_graph_authority,
        result_buffers: Default::default(),
        basis_leases: Default::default(),
        next_external_dispatch_attempt: std::sync::atomic::AtomicU64::new(1),
        external_effect_transport: std::sync::OnceLock::new(),
        recovery_handles,
        mutation_handlers,
        mutation_projection,
        installed_producers: Default::default(),
        output_producer_routes,
        output_readiness_routes,
        next_output_producer_attempt: std::sync::atomic::AtomicU64::new(1),
        next_application_mutation_partition: std::sync::atomic::AtomicU32::new(1),
        output_demands: Default::default(),
        installed_conditionals: Default::default(),
    })
}
