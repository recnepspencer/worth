//! Concrete original publication custody is carried through returning phases.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::managed_index::ManagedIndexProviderCatalog,
    application_resource::installation::PendingResourceRegistry,
    bootstrap::{BootstrapPublicationReservation, BootstrapPublicationStorage},
    handler::InstalledMutationHandlerRegistry,
    WorthQueryApplicationInvariantProjectionAuthority,
};
pub(super) struct PreparedPublication<S: ApplicationSchema> {
    pub(super) input: ApplicationRuntimePublication<S>,
    pub(super) mutation_handlers: InstalledMutationHandlerRegistry<S>,
    pub(super) managed_indexes: ManagedIndexProviderCatalog<S>,
    pub(super) resources: PendingResourceRegistry<S>,
    pub(super) mutation_projection: WorthQueryApplicationInvariantProjectionAuthority<S>,
}
pub(super) struct AuthorizedPublication<S: ApplicationSchema> {
    pub(super) prepared: PreparedPublication<S>,
    pub(super) authorization: WorthQueryInstalledAuthorizationRegistry,
}
#[inline(never)]
fn preflight<S: ApplicationSchema>(
    mut input: ApplicationRuntimePublication<S>,
) -> Result<PreparedPublication<S>, WorthQueryPrimaryGraphInstallationDenial> {
    validate_application_schema(&input.runtime, &input.installed_schema)?;
    let mutation_handlers =
        owned_contributions::preflight(&input.bootstrap, &input.installed_schema)?;
    let managed_indexes = input
        .bootstrap
        .managed_indexes
        .take_complete(&input.installed_schema)?;
    let resources = std::mem::take(&mut input.bootstrap.resources);
    let mutation_projection = input.bootstrap.retain_invariant_projection_authority();
    Ok(PreparedPublication {
        input,
        mutation_handlers,
        managed_indexes,
        resources,
        mutation_projection,
    })
}
#[inline(never)]
pub(super) fn prepare_clock<S: ApplicationSchema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    input: ApplicationRuntimePublication<S>,
) -> Result<
    BootstrapPublicationStorage<PreparedPublication<S>>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let prepared = preflight(input)?;
    Ok(BootstrapPublicationReservation::prepare(&phase.execution_request())?.fill(prepared))
}
#[inline(never)]
pub(super) fn prepare_conditional<S: ApplicationSchema + 'static>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    input: ApplicationRuntimePublication<S>,
    expected: usize,
    bindings: &[Box<
        dyn super::super::super::conditional_operation::WorthQueryPendingConditionalOperation<S>,
    >],
    output_readiness: usize,
    definitions: usize,
) -> Result<
    BootstrapPublicationStorage<PreparedPublication<S>>,
    super::super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
> {
    use super::super::super::conditional_operation::{
        publication_denial, require_complete_binding_inventory,
    };
    let prepared = preflight(input).map_err(publication_denial)?;
    require_complete_binding_inventory(expected, bindings, output_readiness, definitions)?;
    Ok(
        BootstrapPublicationReservation::prepare(&phase.execution_request())
            .map_err(publication_denial)?
            .fill(prepared),
    )
}
#[inline(never)]
pub(super) fn authorize<S: ApplicationSchema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<PreparedPublication<S>>,
) -> Result<
    BootstrapPublicationStorage<AuthorizedPublication<S>>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let prepared = prepared.consume();
    let authorization =
        compile_authorization(&prepared.input.bootstrap, &prepared.input.installed_schema)?;
    Ok(storage.fill(AuthorizedPublication {
        prepared,
        authorization,
    }))
}
