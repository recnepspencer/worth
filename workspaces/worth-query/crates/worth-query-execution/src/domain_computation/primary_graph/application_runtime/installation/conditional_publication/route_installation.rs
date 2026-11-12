//! Seal actual conditional routes and resources before runtime assembly.
use super::*;
#[inline(never)]
pub(super) fn install<S: ApplicationSchema + 'static>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    published: BootstrapPublicationStorage<PublishedConditionalGraph<S>>,
    bindings: Vec<Box<dyn crate::domain_computation::primary_graph::conditional_operation::WorthQueryPendingConditionalOperation<S>>>,
    definitions: Vec<Box<dyn crate::domain_computation::primary_graph::conditional_operation::PendingDefinitionDeclaration>>,
    output_readiness: Vec<Box<dyn crate::domain_computation::primary_graph::application_contribution::PendingOutputReadiness<S>>>,
    producers: &crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationProducerRegistry<S>,
) -> Result<
    BootstrapPublicationStorage<ConfiguredConditionalGraph<S>>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let graph_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let PublishedConditionalGraph {
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        mutation_handlers,
        mutation_projection,
        managed_indexes,
        resources,
    } = published.consume();
    let mut graph = graph.consume();
    let authoritative_commit_cursor = graph
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .latest_position();
    let conditional_operations =
        crate::domain_computation::primary_graph::conditional_operation::install_pending_bindings(
            bindings,
            graph.bridge.conditional_builder(),
            &graph.primary_graph_authority,
            authoritative_commit_cursor,
            graph.runtime.authority_identity().as_u64(),
            graph.runtime.installed_packages().runtime_ordinal(),
            graph.runtime.installed_packages().generation().ordinal(),
            graph.primary_graph_authority.provider_identity(),
            crate::domain_computation::primary_graph::application_branch::PRIMARY_APPLICATION_BRANCH,
        )?;
    let definition_routes = crate::domain_computation::primary_graph::conditional_operation::install_pending_definitions(
        definitions, graph.bridge.conditional_builder(), &graph.primary_graph_authority)?;
    let output_producer_routes = producers
        .install_signal_routes(graph.bridge.conditional_builder())
        .map_err(
            crate::domain_computation::primary_graph::conditional_operation::publication_denial,
        )?;
    let output_readiness_routes =
        crate::domain_computation::primary_graph::application_contribution::install_output_readiness_routes(
            &output_readiness,
            graph.bridge.conditional_builder(),
            &graph.primary_graph_authority,
        )?;
    producers
        .validate_readiness_routes(&output_readiness_routes)
        .map_err(
            crate::domain_computation::primary_graph::conditional_operation::publication_denial,
        )?;
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
        .map_err(
            crate::domain_computation::primary_graph::conditional_operation::publication_denial,
        )?;
    Ok(storage.fill(ConfiguredConditionalGraph {
        graph: graph_storage.fill(graph),
        installed_schema,
        authorization,
        authorization_clock,
        mutation_handlers,
        mutation_projection,
        managed_indexes,
        resources,
        output_producer_routes,
        output_readiness_routes,
        conditional_operations,
        definition_routes,
    }))
}
