//! Complete construction of one contribution-composed in-memory application.

mod denial;
mod limits;
mod program;
pub use denial::WorthQueryInMemoryApplicationDenial;
pub use limits::WorthQueryInMemoryApplicationLimits;
pub use program::{
    in_memory_program, program_execution_port, WorthQueryAdmittedProgramOutput,
    WorthQueryProgramApplicationRuntime, WorthQueryProgramConnectionBindings,
    WorthQueryProgramExecutionPort, WorthQueryProgramOutputAdvance, WorthQuerySettledProgramOutput,
};

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
    in_memory_with_program(declaration, configuration, limits, initial_state, false)
}

pub(super) fn in_memory_with_program<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    allow_program_required: bool,
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
    ));
    let contracts = Schema::Contributions::contracts().map_err(Denial::Contributions)?;
    let package = contracts
        .compose_package(package.application_schema(declaration.clone()))
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
        contracts,
    )
    .map_err(Denial::Contributions)?;
    let (invariants, handlers, producers, conditionals) =
        configured.into_parts().map_err(Denial::Contributions)?;
    if !allow_program_required {
        if let Some(binding) = installed
            .installed_mutation_binding_inventory()
            .find(|binding| binding.requires_application_program())
        {
            return Err(Denial::ApplicationProgramRequired(
                binding.identity().to_owned(),
            ));
        }
    }
    let mut graph = authority
        .prepare_primary_graph_with_invariants(&runtime, &installed, limits.world, invariants)
        .map_err(Denial::Graph)?;
    graph.mutation_handlers = handlers;
    initial_state(&mut graph, &installed).map_err(Denial::InitialState)?;
    let (mut application, installed_conditionals) =
        if conditionals.is_empty() && producers.is_empty() {
            let application = graph
                .publish_application_runtime(runtime, authority, installed, limits.conditionals)
                .map_err(Denial::Publication)?;
            (application, Default::default())
        } else {
            let mut publication = graph
                .conditional_application_runtime_installation(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                )
                .map_err(Denial::ConditionalPublication)?;
            publication.install_output_producers(producers.clone());
            let installed_conditionals = conditionals
                .install_all(&producers, &mut publication)
                .map_err(Denial::ConditionalPublication)?;
            let application = publication
                .publish()
                .map_err(Denial::ConditionalPublication)?;
            (application, installed_conditionals)
        };
    application.installed_producers = producers;
    application.installed_conditionals = installed_conditionals;
    Ok(application)
}
