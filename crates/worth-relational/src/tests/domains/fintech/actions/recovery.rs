use crate::facade::durability::{
    CompactionOutcome, DurabilityError, DurableCheckpoint, RecoveryPlan,
};
use crate::facade::runtime::RelationalRuntime;
use crate::runtime::RecoveryOutcome;

use super::super::fixture::FintechWorld;

pub(crate) fn checkpoint_world(
    world: &mut FintechWorld,
) -> Result<DurableCheckpoint, DurabilityError> {
    world.runtime.durability_authority().checkpoint()
}

pub(crate) fn recover_persisted_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
) -> Result<(RelationalRuntime, RecoveryOutcome), String> {
    recover_runtime_from_plan(
        execution,
        world.runtime.durability().recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        ),
    )
}

pub(crate) fn recover_runtime_from_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
) -> Result<(RelationalRuntime, RecoveryOutcome), String> {
    let mut recovered = RelationalRuntime::new(plan.config.clone());
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .map_err(|error| format!("failed to recover persisted fintech world: {error:?}"))?;
    Ok((recovered, outcome))
}

pub(crate) fn compact_world_store(
    world: &mut FintechWorld,
) -> Result<CompactionOutcome, DurabilityError> {
    world.runtime.durability_authority().compact_store()
}
