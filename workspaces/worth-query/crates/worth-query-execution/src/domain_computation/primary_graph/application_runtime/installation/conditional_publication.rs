//! Assemble conditional service only after its original authorization returns.
use super::*;
pub(super) fn publish<Schema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    authorized: crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage<
        publication_preparation::AuthorizedPublication<Schema>,
    >,
    bindings: Vec<
        Box<
            dyn super::super::super::conditional_operation::WorthQueryPendingConditionalOperation<
                Schema,
            >,
        >,
    >,
    output_readiness: Vec<
        Box<dyn super::super::super::application_contribution::PendingOutputReadiness<Schema>>,
    >,
    producers: &super::super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    super::super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema + 'static,
{
    let publication_preparation::AuthorizedPublication {
        prepared,
        authorization,
    } = authorized.consume();
    let publication_preparation::PreparedPublication {
        input,
        mutation_handlers,
        managed_indexes,
        resources,
        mutation_projection,
    } = prepared;
    let ApplicationRuntimePublication {
        bootstrap,
        runtime,
        authority,
        installed_schema,
        authorization_clock,
        fault_port,
        conditional_evaluation_budget,
    } = input;
    let mut graph = publish_application_graph(
        phase,
        bootstrap,
        runtime,
        authority,
        &installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )
    .map_err(super::super::super::conditional_operation::publication_denial)?;
    let authoritative_commit_cursor = graph
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .latest_position();
    let mut conditional_operations =
        super::super::super::conditional_operation::install_pending_bindings(
            bindings,
            graph.bridge.conditional_builder(),
            &graph.primary_graph_authority,
            authoritative_commit_cursor,
            graph.runtime.authority_identity().as_u64(),
            graph.runtime.installed_packages().runtime_ordinal(),
            graph.runtime.installed_packages().generation().ordinal(),
            graph.primary_graph_authority.provider_identity(),
            super::super::super::application_branch::PRIMARY_APPLICATION_BRANCH,
        )?;
    let output_producer_routes = producers
        .install_signal_routes(graph.bridge.conditional_builder())
        .map_err(super::super::super::conditional_operation::publication_denial)?;
    let output_readiness_routes =
        super::super::super::application_contribution::install_output_readiness_routes(
            output_readiness,
            graph.bridge.conditional_builder(),
            &graph.primary_graph_authority,
        )?;
    producers
        .validate_readiness_routes(&output_readiness_routes)
        .map_err(super::super::super::conditional_operation::publication_denial)?;
    {
        let mut lineage = graph
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        lineage.install_output_families(producers.output_family_bindings());
    }
    let resources = resources
        .install(
            &phase.execution_request(),
            Some(graph.bridge.conditional_builder()),
            graph.primary_provider.application_resource_support_ref(),
        )
        .map_err(super::super::super::conditional_operation::publication_denial)?;
    let graph = seal_application_graph(graph)
        .map_err(super::super::super::conditional_operation::publication_denial)?;
    let mut application = assemble_application_runtime(
        phase,
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
        mutation_handlers,
        mutation_projection,
        output_producer_routes,
        output_readiness_routes,
    )
    .map_err(super::super::super::conditional_operation::publication_denial)?;
    owned_contributions::transfer(&mut application, resources, managed_indexes);
    conditional_operations.reconstruct_all(&application)?;
    conditional_operations
        .reconcile_all(&mut application.runtime.bridge.conditional_lifecycle())?;
    *application
        .conditional_cleanup
        .operations
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = conditional_operations;
    Ok(application)
}
