//! Exclusive Native bootstrap finishes before Query service publication.
use super::*;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::primary_graph::{
    application_checkpoint::DecodedApplicationCheckpoint,
    application_contribution::{
        conditional::PendingConditionalRegistry, WorthQueryInstalledApplicationProducerRegistry,
    },
    application_installation::program_admission::WorthQueryAdmittedProgramSupport,
    bootstrap::{BootstrapPublicationReservation, BootstrapPublicationStorage},
    program_occurrence::WorthQueryProgramActivationCell,
};
pub(super) struct PreparedMemoryApplication<S: ApplicationSchemaComposition> {
    pub(super) graph: WorthQueryPrimaryGraphBootstrap<S>,
    pub(super) runtime: WorthQueryExecutionRuntime,
    pub(super) authority: WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<S>,
    pub(super) producers: WorthQueryInstalledApplicationProducerRegistry<S>,
    pub(super) conditionals: PendingConditionalRegistry<S>,
    pub(super) build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    pub(super) conditional_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    pub(super) authorization_time_source:
        Option<Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>>,
    pub(super) admitted_program_support: Option<WorthQueryAdmittedProgramSupport<S>>,
    pub(super) activation: WorthQueryProgramActivationCell,
    pub(super) decoded_checkpoint: Option<DecodedApplicationCheckpoint>,
    pub(super) interim_continuation:
        Option<worth_relational::facade::durability::NativeInterimBootstrapContinuation>,
}

#[inline(never)]
pub(super) fn prepare_native<Schema, Contributions>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryApplicationLimits,
    initial_state: impl FnOnce(
        &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    program_admission: Option<WorthQueryProgramAdmissionStep<'_, Schema>>,
    checkpoint: Option<super::super::super::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<
        super::super::super::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>,
    >,
) -> Result<
    BootstrapPublicationStorage<PreparedMemoryApplication<Schema>>,
    WorthQueryApplicationPreparationDenial,
>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationPreparationDenial as Denial;
    let execution = phase.execution_request();
    if let Some(checkpoint) = &checkpoint {
        checkpoint
            .preflight_interim(&execution)
            .map_err(|cause| Denial::Graph(cause.into_installation_denial()))?;
    }
    let mut decoded_checkpoint = checkpoint
        .map(|checkpoint| checkpoint.decode_with_execution(&execution))
        .transpose()
        .map_err(|cause| Denial::Graph(cause.into_installation_denial()))?;

    let super::super::prepared_meaning::PreparedInstalledMeaning {
        runtime,
        authority,
        installed,
        admitted_program_support,
        configured,
    } = super::super::prepared_meaning::prepare::<Schema, Contributions>(
        declaration,
        configuration,
        &limits,
        program_admission,
        None,
    )?;
    let (
        invariants,
        super::super::super::application_contribution::PreparedApplicationRuntimeContributions {
            handlers,
            producers,
            conditionals,
            resources,
            managed_indexes,
        },
        activation,
    ) = super::super::contribution_preparation::prepare(
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
                super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
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
        .map_err(|refused| Denial::Contributions(refused.into_cause()))?;
    super::super::source_scope_ownership::seal(
        &graph.graph.layout,
        &mut graph.graph.application_publication,
        &producers,
    )
    .map_err(Denial::Graph)?;
    let mut interim_continuation = None;
    if restoring && let Some(support) = &admitted_program_support {
        if let Some(transition) = checkpoint_transition {
            let (next_graph, continuation) = super::super::super::bootstrap::checkpoint_transition::transition_checkpoint(
                phase,
                graph,
                decoded_checkpoint.as_mut().expect("restoring checkpoint"),
                super::super::super::bootstrap::checkpoint_transition::InstalledCheckpointTarget {
                    schema: &installed,
                    program: support,
                    producers: &producers,
                },
                &activation,
                transition,
            )?;
            graph = next_graph;
            interim_continuation = Some(continuation);
        } else {
            super::super::super::bootstrap::recover_program_activation(
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
    // Codec/program activation may rebind the native publisher. Install the
    // same immutable declaration roster on that final publication authority.
    super::super::source_scope_ownership::seal(
        &graph.graph.layout,
        &mut graph.graph.application_publication,
        &producers,
    )
    .map_err(Denial::Graph)?;
    let storage = BootstrapPublicationReservation::prepare(&execution).map_err(Denial::Graph)?;
    if !restoring {
        if let Some(support) = &admitted_program_support {
            graph.program_activation_seed = Some(
                super::super::super::bootstrap::WorthQueryProgramActivationSeed::for_initial_program(
                    &support.initial_revision,
                    activation.clone(),
                ),
            );
        }
        initial_state(phase, &mut graph, &installed).map_err(Denial::InitialState)?;
    }
    Ok(storage.fill(PreparedMemoryApplication {
        graph,
        runtime,
        authority,
        installed,
        producers,
        conditionals,
        build_identity,
        conditional_budget: limits.conditionals,
        authorization_time_source,
        admitted_program_support,
        activation,
        decoded_checkpoint,
        interim_continuation,
    }))
}
