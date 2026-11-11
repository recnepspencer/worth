use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::durability::DurableCheckpoint;
use crate::facade::runtime::RelationalRuntime;
use crate::runtime::RecoveryOutcome;

use super::super::fixture::FintechWorld;

pub(crate) fn checkpoint_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
    world.runtime.durability_authority().checkpoint(execution)
}

pub(crate) fn recover_checkpointed_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
) -> Result<(RelationalRuntime, RecoveryOutcome), String> {
    let mut recovered = RelationalRuntime::new(world.runtime.config().clone())
        .expect("valid native runtime configuration");
    let plan = world
        .runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .map_err(|error| {
            format!("fintech checkpoint reconstruction refused its owner plan: {error:?}")
        })?;
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .map_err(|error| format!("failed to reconstruct fintech world: {error:?}"))?;
    Ok((recovered, outcome))
}
