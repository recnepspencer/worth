//! Complete construction of one contribution-composed in-memory application.

mod denial;
mod limits;
mod profile;
mod program;
pub use denial::WorthQueryInMemoryApplicationDenial;
pub use limits::WorthQueryInMemoryApplicationLimits;
pub use profile::WorthQueryInMemoryApplicationProfile;
pub use program::{
    in_memory_program, in_memory_program_from_checkpoint,
    in_memory_program_with_authorization_time_source, WorthQueryAdmittedProgramOperation,
    WorthQueryAdmittedProgramOutput, WorthQueryApplicationPreviewReadmissionDenial,
    WorthQueryApplicationPreviewRequest, WorthQueryApplicationPreviewSession,
    WorthQueryApplicationProgramRoots, WorthQueryProgramApplicationRuntime,
    WorthQueryProgramOutputAdvance, WorthQueryProgramRootDemand,
    WorthQueryReadmittedApplicationPreview, WorthQuerySettledProgramOutput,
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

/// Installs contribution-owned schema meaning without an application program.
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
    in_memory_with_contributions::<Schema, Schema::Contributions>(
        declaration,
        configuration,
        limits,
        initial_state,
        None,
        None,
    )
}

pub(super) fn in_memory_with_contributions<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    checkpoint: Option<super::WorthQueryApplicationCheckpoint>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryInMemoryApplicationDenial as Denial;

    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ));
    let contracts = Contributions::contracts().map_err(Denial::Contributions)?;
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
    let configured = WorthQueryConfiguredApplicationContributions::<Schema>::configure::<
        Contributions,
    >(&installed, configuration, contracts)
    .map_err(Denial::Contributions)?;
    let (invariants, handlers, producers, conditionals) =
        configured.into_parts().map_err(Denial::Contributions)?;
    let mut relational_builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .profile(limits.profile.relational_profile());
    if let Some(publication) = limits
        .profile
        .publication_override(limits.maximum_publication_records)
    {
        relational_builder = relational_builder.publication(publication);
    }
    let relational_runtime = relational_builder.build();
    let decoded_checkpoint = checkpoint
        .as_ref()
        .map(super::WorthQueryApplicationCheckpoint::decode)
        .transpose()
        .map_err(|detail| {
            Denial::Graph(WorthQueryPrimaryGraphInstallationDenial::new(
                super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                detail,
            ))
        })?;
    let restoring = decoded_checkpoint.is_some();
    let mut graph = match decoded_checkpoint.as_ref() {
        Some(checkpoint) => authority.prepare_primary_graph_from_native_checkpoint_with_invariants(
            &runtime,
            &installed,
            relational_runtime,
            limits.world,
            invariants,
            checkpoint,
        ),
        None => authority.prepare_primary_graph_with_relational_runtime_and_invariants(
            &runtime,
            &installed,
            relational_runtime,
            limits.world,
            invariants,
        ),
    }
    .map_err(Denial::Graph)?;
    graph.mutation_handlers = handlers;
    if !restoring {
        initial_state(&mut graph, &installed).map_err(Denial::InitialState)?;
    }
    let (mut application, installed_conditionals) =
        if conditionals.is_empty() && producers.is_empty() {
            let application = match authorization_time_source {
                Some(source) => graph.publish_application_runtime_with_authorization_time_source(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                    source,
                ),
                None => graph.publish_application_runtime(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                ),
            }
            .map_err(Denial::Publication)?;
            (application, Default::default())
        } else {
            let mut publication = match authorization_time_source {
                Some(source) => graph
                    .conditional_application_runtime_installation_with_authorization_time_source(
                        runtime,
                        authority,
                        installed,
                        limits.conditionals,
                        source,
                    ),
                None => graph.conditional_application_runtime_installation(
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                ),
            }
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
    application.recovered_outputs = decoded_checkpoint
        .map(|checkpoint| {
            checkpoint
                .accepted_outputs
                .into_iter()
                .map(|accepted| {
                    let correspondence = application
                        .installed_producers
                        .readmit_checkpoint_output(&application.installed_schema, &accepted)
                        .map_err(|detail| {
                            Denial::Graph(WorthQueryPrimaryGraphInstallationDenial::new(
                                super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                                detail,
                            ))
                        })?;
                    Ok(
                        super::application_output_demand::WorthQueryReadmittedAcceptedOutput {
                            checkpoint: accepted,
                            correspondence,
                        },
                    )
                })
                .collect::<Result<Vec<_>, Denial>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(application)
}
