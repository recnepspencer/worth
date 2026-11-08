//! Assemble the default graph only after authorization preparation returns.
use super::*;
pub(super) fn publish<Schema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    authorized: crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage<
        publication_preparation::AuthorizedPublication<Schema>,
    >,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
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
    let graph = publish_application_graph(
        phase,
        bootstrap,
        runtime,
        authority,
        &installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )?;
    let graph_storage = crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let mut graph = graph.consume();
    let resources = resources.install(
        &phase.execution_request(),
        Some(graph.bridge.conditional_builder()),
        graph.primary_provider.application_resource_support_ref(),
    )?;
    let graph = seal_application_graph(phase, graph_storage.fill(graph))?;
    let mut application = assemble_application_runtime(
        phase,
        graph.consume(),
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
        mutation_handlers,
        mutation_projection,
        Default::default(),
        Default::default(),
    )?;
    owned_contributions::transfer(&mut application, resources, managed_indexes);
    Ok(application)
}
