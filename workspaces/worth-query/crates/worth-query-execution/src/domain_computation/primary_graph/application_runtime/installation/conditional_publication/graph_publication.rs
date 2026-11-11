//! Publish from backed inputs, with completion metadata outside the producer frame.
use super::*;
struct GraphMetadata<S: ApplicationSchema> {
    installed_schema: WorthQueryInstalledApplicationSchema<S>,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    mutation_handlers: InstalledMutationHandlerRegistry<S>,
    mutation_projection: WorthQueryApplicationInvariantProjectionAuthority<S>,
    managed_indexes: ManagedIndexProviderCatalog<S>,
    resources: PendingResourceRegistry<S>,
}
struct GraphInput<S: ApplicationSchema> {
    bootstrap: WorthQueryPrimaryGraphBootstrap<S>,
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    fault_port: std::sync::Arc<dyn crate::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFaultPort>,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
}
#[inline(never)]
fn prepare<S: ApplicationSchema>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    authorized: BootstrapPublicationStorage<publication_preparation::AuthorizedPublication<S>>,
) -> Result<
    (
        BootstrapPublicationStorage<GraphInput<S>>,
        BootstrapPublicationStorage<GraphMetadata<S>>,
    ),
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let input_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let metadata_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
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
    Ok((
        input_storage.fill(GraphInput {
            bootstrap,
            runtime,
            authority,
            fault_port,
            conditional_evaluation_budget,
        }),
        metadata_storage.fill(GraphMetadata {
            installed_schema,
            authorization,
            authorization_clock,
            mutation_handlers,
            mutation_projection,
            managed_indexes,
            resources,
        }),
    ))
}
#[inline(never)]
fn produce<S: ApplicationSchema>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    input: BootstrapPublicationStorage<GraphInput<S>>,
    metadata: &GraphMetadata<S>,
) -> Result<
    BootstrapPublicationStorage<PublishedApplicationGraph<WorthQueryApplicationBridgeInstallation>>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let GraphInput {
        bootstrap,
        runtime,
        authority,
        fault_port,
        conditional_evaluation_budget,
    } = input.consume();
    publish_application_graph(
        phase,
        bootstrap,
        runtime,
        authority,
        &metadata.installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )
    .map_err(publication_denial)
}
#[inline(never)]
fn finish<S: ApplicationSchema>(
    storage: BootstrapPublicationReservation<PublishedConditionalGraph<S>>,
    graph: BootstrapPublicationStorage<
        PublishedApplicationGraph<WorthQueryApplicationBridgeInstallation>,
    >,
    metadata: BootstrapPublicationStorage<GraphMetadata<S>>,
) -> BootstrapPublicationStorage<PublishedConditionalGraph<S>> {
    let GraphMetadata {
        installed_schema,
        authorization,
        authorization_clock,
        mutation_handlers,
        mutation_projection,
        managed_indexes,
        resources,
    } = metadata.consume();
    storage.fill(PublishedConditionalGraph {
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        mutation_handlers,
        mutation_projection,
        managed_indexes,
        resources,
    })
}
#[inline(never)]
pub(super) fn publish<S: ApplicationSchema>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    authorized: BootstrapPublicationStorage<publication_preparation::AuthorizedPublication<S>>,
) -> Result<
    BootstrapPublicationStorage<PublishedConditionalGraph<S>>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let (input, metadata) = prepare(phase, authorized)?;
    let graph = produce(phase, input, metadata.value())?;
    Ok(finish(storage, graph, metadata))
}
