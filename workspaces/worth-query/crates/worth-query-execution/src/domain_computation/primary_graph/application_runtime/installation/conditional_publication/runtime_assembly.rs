//! Assemble service after graph publication and route preparation have retired.
use super::*;
#[inline(never)]
pub(super) fn assemble<S: ApplicationSchema>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<PreparedConditionalAssembly<S>>,
) -> Result<
    BootstrapPublicationStorage<UnreconciledConditionalRuntime<S>>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(publication_denial)?;
    let PreparedConditionalAssembly {
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
    } = prepared.consume();
    let mut application = assemble_application_runtime(
        phase,
        graph.consume(),
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
        mutation_handlers,
        mutation_projection,
        output_producer_routes,
        output_readiness_routes,
    )
    .map_err(crate::domain_computation::primary_graph::conditional_operation::publication_denial)?;
    owned_contributions::transfer(&mut application, resources, managed_indexes);
    Ok(storage.fill(UnreconciledConditionalRuntime {
        application,
        conditional_operations,
    }))
}
