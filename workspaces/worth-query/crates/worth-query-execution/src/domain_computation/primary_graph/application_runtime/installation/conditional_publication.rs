//! Assemble conditional service only after its original authorization returns.
use super::*;
use crate::domain_computation::authorization::WorthQueryRuntimeClock;
use crate::domain_computation::primary_graph::{
    application_contribution::{
        WorthQueryInstalledOutputProducerRoutes, WorthQueryInstalledOutputReadinessRoutes,
    },
    application_query::managed_index::ManagedIndexProviderCatalog,
    application_resource::installation::{InstalledResourceRegistry, PendingResourceRegistry},
    bootstrap::{BootstrapPublicationReservation, BootstrapPublicationStorage},
    conditional_operation::{
        publication_denial, WorthQueryConditionalOperationRegistry,
        WorthQueryConditionalRuntimeInstallationDenial,
    },
    handler::InstalledMutationHandlerRegistry,
    managed_bridge::{
        WorthQueryApplicationBridgeInstallation, WorthQueryInstalledApplicationBridge,
    },
    WorthQueryApplicationInvariantProjectionAuthority,
};
mod graph_publication;
mod reconciliation;
mod route_installation;
mod runtime_assembly;
struct PublishedConditionalGraph<S: ApplicationSchema> {
    graph: PublishedApplicationGraph<WorthQueryApplicationBridgeInstallation>,
    installed_schema: WorthQueryInstalledApplicationSchema<S>,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    mutation_handlers: InstalledMutationHandlerRegistry<S>,
    mutation_projection: WorthQueryApplicationInvariantProjectionAuthority<S>,
    managed_indexes: ManagedIndexProviderCatalog<S>,
    resources: PendingResourceRegistry<S>,
}
struct PreparedConditionalAssembly<S: ApplicationSchema> {
    graph: PublishedApplicationGraph<WorthQueryInstalledApplicationBridge>,
    installed_schema: WorthQueryInstalledApplicationSchema<S>,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    mutation_handlers: InstalledMutationHandlerRegistry<S>,
    mutation_projection: WorthQueryApplicationInvariantProjectionAuthority<S>,
    managed_indexes: ManagedIndexProviderCatalog<S>,
    resources: InstalledResourceRegistry<S>,
    output_producer_routes: WorthQueryInstalledOutputProducerRoutes,
    output_readiness_routes: WorthQueryInstalledOutputReadinessRoutes,
    conditional_operations: WorthQueryConditionalOperationRegistry<S>,
}
struct UnreconciledConditionalRuntime<S: ApplicationSchema> {
    application: WorthQueryPrimaryGraphApplicationRuntime<S>,
    conditional_operations: WorthQueryConditionalOperationRegistry<S>,
}

#[inline(never)]
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
    let graph = graph_publication::publish(phase, authorized)?;
    let assembly =
        route_installation::install(phase, graph, bindings, output_readiness, producers)?;
    let runtime = runtime_assembly::assemble(phase, assembly)?;
    reconciliation::complete(runtime)
}
