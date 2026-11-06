//! Publish the original graph before installing conditional routes.
use super::*;
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
    )
    .map_err(crate::domain_computation::primary_graph::conditional_operation::publication_denial)?;
    Ok(storage.fill(PublishedConditionalGraph {
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        mutation_handlers,
        mutation_projection,
        managed_indexes,
        resources,
    }))
}
