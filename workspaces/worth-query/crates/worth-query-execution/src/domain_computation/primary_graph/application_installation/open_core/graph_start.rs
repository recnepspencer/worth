//! Prepare the graph and start its home, retaining adoption custody.
use super::super::{
    home_opening::HomeStarted,
    home_start::{self, AdmittedEntry},
    open_plan::HomeStart,
    open_refusal::OpenFailure,
    WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial as Denial,
};
use super::backend_construction::BackendInstallation;
use super::contribution_configuration::ConfiguredInstallation;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::primary_graph::{
    application_contribution::{PendingConditionalRegistry, PendingProducerRegistry},
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    program_occurrence::WorthQueryProgramActivationCell,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

pub(super) struct StartedInstallation<Schema: ApplicationSchema> {
    pub(super) runtime: WorthQueryExecutionRuntime,
    pub(super) authority: WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<Schema>,
    pub(super) entry: AdmittedEntry<Schema>,
    pub(super) activation: WorthQueryProgramActivationCell,
    pub(super) graph: WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(super) producers: PendingProducerRegistry<Schema>,
    pub(super) conditionals: PendingConditionalRegistry<Schema>,
    pub(super) started: HomeStarted,
    pub(super) accepted_outputs: Vec<WorthQueryAcceptedOutputCheckpointIdentity>,
}

pub(super) fn start<Schema: ApplicationSchema>(
    backend: BackendInstallation<Schema>,
    limits: &WorthQueryApplicationLimits,
    start: HomeStart<'_, Schema>,
) -> Result<StartedInstallation<Schema>, OpenFailure> {
    let BackendInstallation {
        configured,
        relational_runtime,
    } = backend;
    let ConfiguredInstallation {
        runtime,
        authority,
        installed,
        entry,
        activation,
        invariants,
        handlers,
        producers,
        conditionals,
    } = configured;
    let (graph, started, accepted_outputs) = match start {
        HomeStart::Empty { initial_state } => {
            let mut graph = authority
                .prepare_primary_graph_with_relational_runtime_and_invariants(
                    &runtime,
                    &installed,
                    relational_runtime,
                    limits.world.clone(),
                    invariants,
                )
                .map_err(Denial::Graph)?;
            graph.mutation_handlers = handlers;
            home_start::seed_empty(&mut graph, &installed, &entry, &activation, initial_state)?;
            (graph, HomeStarted::Empty, Vec::new())
        }
        HomeStart::Resume { image, adoption } => {
            let image = image.decode().map_err(|detail| {
                Denial::Graph(
                    WorthQueryPrimaryGraphInstallationDenial::checkpoint_recovery_rejected(detail),
                )
            })?;
            let mut graph = authority
                .prepare_primary_graph_from_native_checkpoint_with_invariants(
                    &runtime,
                    &installed,
                    relational_runtime,
                    limits.world.clone(),
                    invariants,
                    &image,
                )
                .map_err(Denial::Graph)?;
            graph.mutation_handlers = handlers;
            let started = home_start::resume(
                &mut graph,
                &installed,
                &entry,
                &activation,
                adoption,
                !image.accepted_outputs.is_empty(),
            )?;
            (graph, started, image.accepted_outputs)
        }
    };
    Ok(StartedInstallation {
        runtime,
        authority,
        installed,
        entry,
        activation,
        graph,
        producers,
        conditionals,
        started,
        accepted_outputs,
    })
}
