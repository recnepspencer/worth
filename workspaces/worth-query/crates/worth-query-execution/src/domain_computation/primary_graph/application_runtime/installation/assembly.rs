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
    // Output-lineage history keeps the generations the invalidation window keeps.
    let history_positions = std::num::NonZeroUsize::new(
        graph
            .product_world_resources
            .invalidation_resources()
            .installation()
            .maximum_retained_positions,
    )
    .expect("installed invalidation resources retain at least one position");
    let product_runtime = crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime::install(
        graph.primary_provider.graph.prepare_product_source(&graph.relational_branch_identity)
            .map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
                format!("Product source admission: {denial:?}"),
            ))?,
        &mut graph.bridge.conditional_lifecycle(),
        graph.product_world_resources,
        graph.recovered_relational_authority,
    ).map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        denial.detail(),
    ))?;
    graph
        .primary_provider
        .install_world_history(product_runtime.owner.lifecycle_port());
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
    let program_required_bindings = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_application_program())
        .map(|binding| binding.binding_type())
        .collect();
    let program_required_operations = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_application_program())
        .map(|binding| binding.operation_type())
        .collect();
    let workflow_guarded_operations = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_workflow_authority())
        .map(|binding| binding.operation_type())
        .collect();
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
    let obligation_budget = graph
        .runtime
        .output_demand_resource_profile()
        .registry_obligation_retained_bytes();
    let required_budget = graph
        .runtime
        .output_demand_resource_profile()
        .registry_required_retained_bytes();
    let record_budget = graph
        .runtime
        .output_demand_resource_profile()
        .registry_record_retained_bytes();
    graph
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .install_lineage_retention(
            graph
                .runtime
                .output_demand_resource_profile()
                .lineage_retained_bytes(),
            history_positions,
        );
    Ok(WorthQueryPrimaryGraphApplicationRuntime {
        runtime: graph.runtime,
        installed_schema,
        application_readiness_schema_token,
        publication: graph.publication,
        checkpoint_restore_work: graph.checkpoint_restore_work,
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
        source_meanings: crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::new(
            runtime_authority,
        ),
        basis_leases: Default::default(),
        next_external_dispatch_attempt: std::sync::atomic::AtomicU64::new(1),
        external_effect_transport: std::sync::OnceLock::new(),
        inbound_verifiers: std::sync::Mutex::new(std::collections::BTreeMap::new()),
        inbound_custody: std::sync::Arc::new(std::sync::Mutex::new(Default::default())),
        transport_completion_custody: std::sync::Mutex::new(Default::default()),
        inbound_maintenance_turn: std::sync::atomic::AtomicU64::new(0),
        recovery_handles,
        mutation_handlers,
        mutation_projection,
        installed_producers: Default::default(),
        output_producer_routes,
        output_readiness_routes,
        next_output_producer_attempt: std::sync::atomic::AtomicU64::new(1),
        next_application_mutation_partition: std::sync::atomic::AtomicU32::new(1),
        output_demands: super::super::super::application_output_demand::WorthQueryOutputDemandRegistry::with_budgets(obligation_budget, record_budget, required_budget),
        recovered_outputs: Default::default(),
        program_required_bindings,
        program_required_operations,
        workflow_guarded_operations,
        program_support: None,
        installed_conditionals: Default::default(),
        workflow_coverage: Default::default(),
    })
}
