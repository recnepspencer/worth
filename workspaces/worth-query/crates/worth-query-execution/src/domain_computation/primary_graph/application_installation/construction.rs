//! Native contribution construction within already admitted bootstrap custody.
use super::super::application_contribution::WorthQueryApplicationContributionTuple;
use super::super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use super::{
    WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
    WorthQueryProgramAdmissionStep,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
pub(super) fn in_memory_with_contributions_in_advancement<Schema, Contributions>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    program_admission: Option<WorthQueryProgramAdmissionStep<'_, Schema>>,
    checkpoint: Option<super::super::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<
        super::super::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>,
    >,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryInMemoryApplicationDenial as Denial;

    let super::prepared_meaning::PreparedInstalledMeaning {
        runtime,
        authority,
        installed,
        admitted_program_support,
        configured,
    } = super::prepared_meaning::prepare::<Schema, Contributions>(
        declaration,
        configuration,
        &limits,
        program_admission,
        None,
    )?;
    let (
        invariants,
        super::super::application_contribution::PreparedApplicationRuntimeContributions {
            handlers,
            producers,
            conditionals,
            resources,
            managed_indexes,
        },
        activation,
    ) = super::contribution_preparation::prepare(
        configured,
        &runtime,
        &installed,
        &admitted_program_support,
    )?;
    let mut relational_builder = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .profile(limits.profile.relational_profile());
    if let Some(publication) = limits
        .profile
        .publication_override(limits.maximum_publication_records)
    {
        relational_builder = relational_builder.publication(publication);
    }
    let relational_runtime = relational_builder
        .build()
        .map_err(|cause| Denial::Graph(cause.into()))?;
    let execution = phase.execution_request();
    let mut decoded_checkpoint = checkpoint
        .map(|checkpoint| checkpoint.decode_with_execution(&execution))
        .transpose()
        .map_err(|cause| Denial::Graph(cause.into_installation_denial()))?;
    let restoring = decoded_checkpoint.is_some();
    let mut graph = authority
        .prepare_native_graph(
            phase,
            &runtime,
            &installed,
            relational_runtime,
            invariants,
            decoded_checkpoint.as_ref(),
            checkpoint_transition.is_some(),
        )
        .map_err(Denial::Graph)?;
    if checkpoint_transition.is_some()
        && decoded_checkpoint
            .as_ref()
            .is_some_and(|checkpoint| !checkpoint.accepted_outputs.is_empty())
        && !graph.has_pending_codec_transition()
    {
        return Err(Denial::Graph(
            WorthQueryPrimaryGraphInstallationDenial::new(
                super::super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                "checkpoint transition requires accepted-output migration support",
            ),
        ));
    }
    // Seal the real configured executors before a recovered producer route can
    // justify retiring an old output. The same registry is published below.
    let producers = producers
        .seal_with_support(
            build_identity,
            graph.resource_support_ref(),
            runtime.installed_packages(),
        )
        .map_err(Denial::Contributions)?;
    if restoring && let Some(support) = &admitted_program_support {
        if let Some(transition) = checkpoint_transition {
            graph = super::super::bootstrap::checkpoint_transition::transition_checkpoint(
                phase,
                graph,
                decoded_checkpoint.as_mut().expect("restoring checkpoint"),
                super::super::bootstrap::checkpoint_transition::InstalledCheckpointTarget {
                    schema: &installed,
                    program: support,
                    producers: &producers,
                },
                &activation,
                transition,
            )?;
        } else {
            super::super::bootstrap::recover_program_activation(
                phase,
                &graph.graph,
                &support.roster,
                &activation,
            )
            .map_err(Denial::Graph)?;
        }
    }
    // No source, Bridge registry or projection handle exists until all exclusive
    // schema and legacy codec transition work above has completed.
    let mut graph = graph
        .into_bootstrap(&runtime, &installed, limits.world)
        .map_err(Denial::Graph)?;
    graph.mutation_handlers = handlers;
    graph.resources = resources;
    graph.managed_indexes = managed_indexes;
    graph.program_activation = admitted_program_support
        .as_ref()
        .map(|_| activation.clone());
    if !restoring {
        if let Some(support) = &admitted_program_support {
            graph.program_activation_seed = Some(
                super::super::bootstrap::WorthQueryProgramActivationSeed::for_initial_program(
                    &support.initial_revision,
                    activation.clone(),
                ),
            );
        }
        initial_state(phase, &mut graph, &installed).map_err(Denial::InitialState)?;
    }
    let (mut application, installed_conditionals) =
        if conditionals.is_empty() && producers.is_empty() {
            let application = match authorization_time_source {
                Some(source) => graph.publish_application_runtime_with_authorization_time_source(
                    phase,
                    runtime,
                    authority,
                    installed,
                    limits.conditionals,
                    source,
                ),
                None => graph.publish_application_runtime(
                    phase,
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
                        phase,
                        runtime,
                        authority,
                        installed,
                        limits.conditionals,
                        source,
                    ),
                None => graph.conditional_application_runtime_installation(
                    phase,
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
                .publish_contributions(phase, &installed_conditionals)
                .map_err(Denial::ConditionalPublication)?;
            (application, installed_conditionals)
        };
    application.installed_producers = producers;
    application.installed_conditionals = installed_conditionals;
    application
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .bind_implementation_build(build_identity);
    if let Some(support) = admitted_program_support {
        application
            .product_runtime
            .activations
            .require_program_coordination();
        application.program_support = Some(
            super::super::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                support.roster,
                activation,
            ),
        );
    }
    if let Some(checkpoint) = decoded_checkpoint {
        application.recovered_outputs =
            super::super::application_output_demand::WorthQueryRecoveredOutputs::from_checkpoint(
                checkpoint,
                &execution,
                |accepted| {
                    application.installed_producers.readmit_checkpoint_output(
                        &application.installed_schema,
                        accepted,
                        &execution,
                    )
                },
            )
            .map_err(Denial::Graph)?;
    }
    super::checkpoint_lineage::restore(&application, &execution).map_err(Denial::Graph)?;
    Ok(application)
}
