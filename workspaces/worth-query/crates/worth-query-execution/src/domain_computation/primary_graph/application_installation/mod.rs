//! Complete construction of one contribution-composed in-memory application.

mod denial;
mod limits;
pub use denial::WorthQueryInMemoryApplicationDenial;
pub use limits::WorthQueryInMemoryApplicationLimits;

use super::application_contribution::{
    WorthQueryApplicationContributionTuple, WorthQueryConfiguredApplicationContributions,
};
use super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledApplicationSchema, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

/// Installs the declaration's contribution configuration and publishes its initial state.
///
/// Configuration is paired with the contribution tuple declared by `Schema`. All
/// handler and invariant factories must be complete before `initial_state` runs.
/// The initializer borrows only the unpublished typed graph and its installed
/// schema. Any error drops construction without exposing a runtime.
pub fn in_memory<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryInMemoryApplicationDenial as Denial;

    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .map_err(Denial::Package)?;
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "primary-graph-in-memory",
        "application-contributions",
    )
    .admit(package)
    .map_err(Denial::Admission)?;
    let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
        .application_candidate_resources(limits.candidates)
        .application_query_resources(limits.queries)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .map_err(Denial::Runtime)?
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .map_err(Denial::Schema)?;
    let configured = WorthQueryConfiguredApplicationContributions::<Schema>::configure(
        &installed,
        configuration,
    )
    .map_err(Denial::Contributions)?;
    let (invariants, handlers) = configured.into_parts();
    let mut graph = authority
        .prepare_primary_graph_with_invariants(&runtime, &installed, limits.world, invariants)
        .map_err(Denial::Graph)?;
    graph.mutation_handlers = handlers;
    initial_state(&mut graph, &installed).map_err(Denial::InitialState)?;
    graph
        .publish_application_runtime(runtime, authority, installed, limits.conditionals)
        .map_err(Denial::Publication)
}
