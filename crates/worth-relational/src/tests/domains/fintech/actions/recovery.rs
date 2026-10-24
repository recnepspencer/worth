use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::durability::DurableCheckpoint;
use crate::facade::runtime::RelationalRuntime;
use crate::runtime::RecoveryOutcome;

use super::super::fixture::FintechWorld;

pub(crate) fn checkpoint_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &world.runtime)?;
    if crate::tests::support::attached_durable_medium(&world.runtime).is_some() {
        crate::tests::support::checkpoint_into_medium(execution, &world.runtime);
    }
    Ok(checkpoint)
}

pub(crate) fn recover_persisted_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
) -> Result<(RelationalRuntime, RecoveryOutcome), String> {
    let mut recovered = RelationalRuntime::new(world.runtime.config().clone());
    let plan = crate::tests::support::medium_recovery_plan(
        execution,
        &world.runtime,
        &mut recovered,
        RecoveryMode::NormalRecoveryVerification,
    )
    .map_err(|error| format!("persisted fintech world refused its medium: {error:?}"))?;
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .map_err(|error| format!("failed to recover persisted fintech world: {error:?}"))?;
    Ok((recovered, outcome))
}
