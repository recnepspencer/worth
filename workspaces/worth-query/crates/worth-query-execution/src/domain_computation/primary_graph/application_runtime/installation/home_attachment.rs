//! Preflight the same application assembly while Host retains bootstrap custody.
use super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge;
use super::*;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

pub(in crate::domain_computation::primary_graph) fn preflight_home_application<
    Schema: ApplicationSchema,
>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    bootstrap: &WorthQueryPrimaryGraphBootstrap<Schema>,
    runtime: &WorthQueryExecutionRuntime,
    authority: &WorthQueryExecutionInstallationAuthority,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    (
        std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        WorthQueryInstalledGraphParticipationAuthority,
        WorthQueryInstalledAuthorizationRegistry,
        super::super::super::handler::InstalledMutationHandlerRegistry<Schema>,
    ),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    validate_application_schema(runtime, installed)?;
    bootstrap.resources.validate_complete(installed)?;
    let handlers = bootstrap
        .mutation_handlers
        .seal(installed, bootstrap.graph.binding_identity())?;
    let authorization = compile_authorization(bootstrap, installed)?;
    let graph = bootstrap.graph.integration_handle();
    graph
        .bind_current_truth_head(
            &phase.execution_request(),
            &super::super::super::application_branch::primary_relational_branch_id(),
        )
        .map_err(|cause| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
                format!("Home truth head admission denied: {cause:?}"),
            )
        })?;
    let truth_partition_role = graph.truth_partition_role().cloned();
    let (anchor, provider) = WorthQueryPrimaryGraphProvider::install(
        graph,
        super::super::super::provider::fault_port::production_fault_port(),
        runtime
            .application_query_resource_profile()
            .maximum_concurrent_graph_work(),
        bootstrap.resource_support.clone(),
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
    Ok((provider, participation, authorization, handlers))
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
    bridge: WorthQueryInstalledApplicationBridge,
    product: WorthQueryProductRuntime,
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
    assembly::assemble_application_runtime_with_product(
        graph,
        installed,
        authorization,
        authorization_clock,
        Default::default(),
        handlers,
        projection,
        Default::default(),
        Default::default(),
        product,
        None,
    )
}
