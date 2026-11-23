//! Restored C2.3 private capture-custody campaigns over the testkit medium.
//! Their originals filed captures into the retired in-crate test medium; here
//! the runtime owns a `TestDurableMedium` attachment and reads its owner only
//! through public facades and counted read-only observations.
mod delayed_selection;
mod selected_ack;

use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        RelationalCheckpointArtifactPolicy, RelationalDurableColdSource,
        RelationalDurableMemberLocator, RelationalNativeCheckpoint,
    },
    runtime::RelationalRuntime,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

/// The original capture: external members, carrying `current_members`, with
/// the runtime's own medium as the retained read port.
fn owned_capture(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    current_members: &[RelationalDurableMemberLocator],
) -> RelationalNativeCheckpoint {
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members,
                source: Some(source),
            },
        )
        .unwrap()
}

fn acknowledge(execution: &ExecutionRequest<'_, '_>, checkpoint: &RelationalNativeCheckpoint) {
    execution
        .run_native_with_checkpoint(|context| checkpoint.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap();
}
