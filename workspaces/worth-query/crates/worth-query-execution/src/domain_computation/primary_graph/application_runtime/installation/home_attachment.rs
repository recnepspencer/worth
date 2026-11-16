//! Preflight the same application assembly while Host retains bootstrap custody.
use super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge;
use super::super::mutation_partition::ApplicationMutationPartitions;
use super::*;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

pub(in crate::domain_computation::primary_graph) struct HomeApplicationServices<
    Schema: ApplicationSchema,
> {
    pub(in crate::domain_computation::primary_graph) provider:
        std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    pub(in crate::domain_computation::primary_graph) participation:
        WorthQueryInstalledGraphParticipationAuthority,
    pub(in crate::domain_computation::primary_graph) authorization:
        WorthQueryInstalledAuthorizationRegistry,
    pub(in crate::domain_computation::primary_graph) handlers:
        super::super::super::handler::InstalledMutationHandlerRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph) resources:
        super::super::super::application_resource::installation::InstalledResourceRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph) mutation_partitions:
        ApplicationMutationPartitions,
}

pub(in crate::domain_computation::primary_graph) fn prepare_home_application_services<
    Schema: ApplicationSchema,
>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    runtime: &WorthQueryExecutionRuntime,
    authority: &WorthQueryExecutionInstallationAuthority,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<HomeApplicationServices<Schema>, WorthQueryPrimaryGraphInstallationDenial> {
    if !bootstrap.product_world_resources.admits_clock(
        &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    ) {
        return Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::AdmissionClockMismatch,
            "Home Product World and admission require the same monotonic clock",
        ));
    }
    validate_application_schema(runtime, installed)?;
    let handlers = super::owned_contributions::preflight(bootstrap, installed)?;
    let authorization = compile_authorization(bootstrap, installed)?;
    let graph = bootstrap.graph.integration_handle();
    let mutation_partitions = graph
        .with_runtime(&phase.execution_request(), |native| {
            ApplicationMutationPartitions::readmit(native, &phase.execution_request())
        })
        .map_err(super::super::super::bootstrap::request_read_denial::read)?;
    let truth_partition_role = graph.truth_partition_role().cloned();
    let (anchor, provider) = WorthQueryPrimaryGraphProvider::install(
        graph,
        super::super::super::provider::fault_port::production_fault_port(),
        runtime
            .application_query_resource_profile()
            .maximum_concurrent_graph_work(),
        bootstrap.resource_support.clone(),
        installed.installed_view().retain_operation_catalog(),
    );
    provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .install_binding_identities(
            installed
                .installed_mutation_binding_inventory()
                .map(|binding| (TypeId::from(binding.binding_type()), binding.identity())),
        );
    let participation =
        super::super::graph_participation::install(authority, truth_partition_role, anchor)?;
    Ok(HomeApplicationServices {
        provider,
        participation,
        authorization,
        handlers,
        resources: Default::default(),
        mutation_partitions,
    })
}

pub(in crate::domain_computation::primary_graph) fn finish_home_application<
    Schema: ApplicationSchema,
>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
    mut runtime: WorthQueryExecutionRuntime,
    installed: WorthQueryInstalledApplicationSchema<Schema>,
    publication: super::super::super::WorthQueryPrimaryGraphPublication,
    provider: std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    participation: WorthQueryInstalledGraphParticipationAuthority,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    handlers: super::super::super::handler::InstalledMutationHandlerRegistry<Schema>,
    resources: super::super::super::application_resource::installation::InstalledResourceRegistry<
        Schema,
    >,
    managed_indexes: super::super::super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>,
    bridge: WorthQueryInstalledApplicationBridge,
    product: WorthQueryProductRuntime,
    mutation_partitions: ApplicationMutationPartitions,
    producers: super::super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
    conditionals: super::super::super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    producer_routes: super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes,
    readiness_routes: super::super::super::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
    definition_routes: crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
) -> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    let projection = bootstrap.retain_invariant_projection_authority();
    let checkpoint_restore_work = bootstrap.recovered_checkpoint_restore_work();
    let product_world_resources = bootstrap.product_world_resources.clone();
    let relational_branch_identity = provider
        .graph
        .with_runtime(&phase.execution_request(), |native| {
            native.main_branch_identity()
        });
    runtime.install_primary_graph(bootstrap.graph);
    let graph = PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge,
        primary_provider: provider,
        primary_graph_authority: participation,
        product_world_resources,
        recovered_relational_authority: None,
        checkpoint_restore_work,
    };
    let mut application = assembly::assemble_application_runtime_with_product(
        graph,
        installed,
        authorization,
        authorization_clock,
        Default::default(),
        handlers,
        projection,
        producer_routes,
        readiness_routes,
        product,
        None,
        mutation_partitions,
    );
    application.installed_producers = producers;
    application.installed_conditionals = conditionals;
    application.conditional_cleanup.definitions = definition_routes;
    super::owned_contributions::transfer(&mut application, resources, managed_indexes);
    application
}

pub(in crate::domain_computation::primary_graph) fn readmit_home_branch_basis(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    provider: &WorthQueryPrimaryGraphProvider,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let graph = &provider.graph;
    graph.with_runtime(&phase.execution_request(), |native| {
        graph.carry_primary_index_basis_for_basis(&phase.execution_request(), native, basis)
            .map_err(|cause| match cause {
                super::super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(cause) =>
                    super::super::super::bootstrap::request_read_denial::read(cause),
                cause => WorthQueryPrimaryGraphInstallationDenial::new(
                    WorthQueryPrimaryGraphInstallationDenialKind::IndexBuildRejected,
                    format!("Home selected-basis index admission denied: {cause:?}")),
            })
    })?;
    Ok(())
}
pub(in crate::domain_computation::primary_graph) fn bind_home_root_basis(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    provider: &WorthQueryPrimaryGraphProvider,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let graph = &provider.graph;
    graph.with_runtime(&phase.execution_request(), |native| {
        graph
            .bind_truth_head_basis_in_runtime(native, basis)
            .map_err(|cause| {
                WorthQueryPrimaryGraphInstallationDenial::new(
                    WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
                    format!("Home truth head admission denied: {cause:?}"),
                )
            })
    })?;
    Ok(())
}
