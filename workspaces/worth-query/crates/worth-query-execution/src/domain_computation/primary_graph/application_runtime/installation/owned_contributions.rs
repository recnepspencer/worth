//! Both recipes validate and transfer the same installed contribution custody.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::managed_index::ManagedIndexProviderCatalog,
    application_resource::installation::InstalledResourceRegistry,
    handler::InstalledMutationHandlerRegistry,
};

pub(super) fn preflight<Schema: ApplicationSchema>(
    bootstrap: &WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<InstalledMutationHandlerRegistry<Schema>, WorthQueryPrimaryGraphInstallationDenial> {
    bootstrap.resources.validate_complete(installed)?;
    bootstrap.managed_indexes.validate_complete(installed)?;
    bootstrap
        .mutation_handlers
        .seal(installed, bootstrap.graph.binding_identity())
}

pub(super) fn transfer<Schema: ApplicationSchema>(
    application: &mut WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    resources: InstalledResourceRegistry<Schema>,
    indexes: ManagedIndexProviderCatalog<Schema>,
) {
    application.conditional_cleanup.resources = resources;
    application.managed_indexes = indexes;
}
