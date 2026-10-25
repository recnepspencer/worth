//! Durable branch lifecycle admission, interruption and reconstruction.

mod checkpoint_state;
mod fates;
mod recovery;
mod replay;
mod tampering;

use crate::branch::{
    RelationalBranchTransitionSequence as Sequence, RelationalBranchTransitionStop as Stop,
};
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact as Artifact;
use crate::facade::history::BranchId;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};

fn fork(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(name.into()), basis)
        .unwrap();
}

fn artifacts(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect()
}

fn reopen(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> RelationalRuntime {
    recover_with(execution, runtime, durable_runtime_with_test_schema).1
}
