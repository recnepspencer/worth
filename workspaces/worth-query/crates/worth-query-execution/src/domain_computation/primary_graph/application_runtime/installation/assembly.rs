use super::PublishedApplicationGraph;
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
pub(super) fn assemble_application_runtime<Schema>(
    phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<'_>,
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
    if !graph
        .product_world_resources
        .admits_clock(&authorization_clock.admission_clock)
    {
        return Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::AdmissionClockMismatch,
            "Product World and admission require the same monotonic clock",
        ));
    }
    let product_runtime = crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime::install(
        graph.primary_provider.graph.prepare_product_source(&phase.execution_request(), &graph.relational_branch_identity)
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
        .install_world_history(&product_runtime);
    let runtime_authority = graph.runtime.authority_identity();
    let schema_binding = installed_schema.binding_identity();
    let application_readiness_schema_token = format!(
        "{}:{}:{}",
        schema_binding.generation(),
        schema_binding.package_identity().render_hex(),
        schema_binding.schema_identity().render_hex(),
    );
    let granular_invalidation = crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation::new(schema_binding.clone(), graph.primary_provider.graph.clone(), crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot::new(
            product_runtime.clone(),
            graph.bridge.conditional_operations(),
        ));
    let program_required_bindings = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_application_program())
        .map(|binding| TypeId::from(binding.binding_type()))
        .collect();
    let program_required_operations = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_application_program())
        .map(|binding| TypeId::from(binding.operation_type()))
        .collect();
    let workflow_guarded_operations = installed_schema
        .installed_mutation_binding_inventory()
        .filter(|binding| binding.requires_workflow_authority())
        .map(|binding| TypeId::from(binding.operation_type()))
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
    let installed_operations = installed_schema.installed_view().retain_operation_catalog();
    Ok(WorthQueryPrimaryGraphApplicationRuntime {
        installed_schema,
        conditional_operations: std::sync::Mutex::new(conditional_operations),
        mutation_handlers,
        mutation_projection,
        output_production: crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputProduction::new(
            output_producer_routes,
            output_readiness_routes,
        ),
        post_conditional_teardown: Default::default(),
        workflow_teardown: Default::default(),
        program_required_bindings,
        program_required_operations,
        workflow_guarded_operations,
        program_support: None,
        installed_conditionals: Default::default(),
        installed_resources: Default::default(),
        managed_indexes: Default::default(),
        signing_owners: Default::default(),
        runtime: super::super::WorthQueryApplicationRuntimeCore {
            execution: Some(graph.runtime),
            application_readiness_schema_token,
            publication: graph.publication,
            checkpoint_restore_work: graph.checkpoint_restore_work,
            authorization: std::sync::Arc::new(authorization),
            authorization_clock,
            relational_branch_identity: graph.relational_branch_identity,
            bridge: graph.bridge,
            product_runtime,
            granular_invalidation,
            primary_provider: graph.primary_provider,
            primary_graph_authority: Some(graph.primary_graph_authority),

            source_meanings: Some(crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::new(
                runtime_authority,
            )),
            recovered_program_source_delivery: Default::default(),
            basis_leases: Default::default(),
            next_external_dispatch_attempt: std::sync::atomic::AtomicU64::new(1),
            external_effect_transport: std::sync::OnceLock::new(),
            inbound_verifiers: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            inbound_custody: std::sync::Arc::new(std::sync::Mutex::new(Default::default())),
            transport_completion_custody: std::sync::Mutex::new(Default::default()),
            inbound_maintenance_turn: std::sync::atomic::AtomicU64::new(0),
            recovery_handles,
            next_application_mutation_partition: std::sync::atomic::AtomicU32::new(1),
            workflow_coverage: Default::default(),
            installed_operations,
        },
    })
}
