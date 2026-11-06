//! Seal only after route configuration's raw successor frame has retired.
use super::*;
#[inline(never)]
pub(super) fn seal<S: ApplicationSchema>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    configured: BootstrapPublicationStorage<ConfiguredConditionalGraph<S>>,
) -> Result<
    BootstrapPublicationStorage<PreparedConditionalAssembly<S>>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let ConfiguredConditionalGraph {
        graph,
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
    } = configured.consume();
    let graph = seal_application_graph(phase, graph).map_err(publication_denial)?;
    Ok(storage.fill(PreparedConditionalAssembly {
        graph,
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
    }))
}
