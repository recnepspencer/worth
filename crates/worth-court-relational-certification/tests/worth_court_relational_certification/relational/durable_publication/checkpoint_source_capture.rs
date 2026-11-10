//! Logical capture and filing on the fixture's explicitly held original medium.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{DurableCheckpoint, RelationalRecoveryStop},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

pub(super) fn capture_durably(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
) -> Result<DurableCheckpoint, RelationalRecoveryStop> {
    let checkpoint = runtime.durability_authority().checkpoint(request)?;
    super::branch_history_recovery::checkpoint(request, runtime, medium);
    Ok(checkpoint)
}
