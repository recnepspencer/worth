//! Observations derive from acknowledged prefixes and the SAME pending attempt.
use super::{
    owner::HomeCheckpointOwner, ApplicationHomeCheckpointDenial as Denial,
    ApplicationHomeCheckpointJournalObservation, ApplicationHomeCheckpointLimits,
    ApplicationHomeCheckpointRecovery,
};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::WorldCheckpointPort;
use worth_store::physical_runtime::ServingPhysicalRuntime;

#[derive(Debug)]
pub struct ApplicationHomeCheckpointInspection {
    pub journals: [ApplicationHomeCheckpointJournalObservation; 5],
    /// Actual canonical bodies from journal transport and selected checkpoint members.
    pub native_canonical_body_reads: u64,
    pub due: bool,
    pub pending: Option<ApplicationHomeCheckpointRecovery>,
}

pub(crate) fn inspect(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    world: &WorldCheckpointPort,
    limits: ApplicationHomeCheckpointLimits,
    native_canonical_body_reads: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ApplicationHomeCheckpointInspection, Denial> {
    request.consult().map_err(Denial::Execution)?;
    let native = owner.acknowledged_native();
    let world = world.acknowledged_positions().map_err(Denial::World)?;
    let cadence = super::cadence::inspect(
        serving,
        [native[0], native[1], world[0], world[1], world[2]],
        limits,
        request,
    )?;
    Ok(ApplicationHomeCheckpointInspection {
        journals: cadence.journals,
        native_canonical_body_reads,
        due: cadence.due,
        pending: owner.pending().map(|attempt| attempt.observation()),
    })
}
