//! Durable branch lifecycle admission, interruption and reconstruction.

mod reconstruction;

use crate::facade::history::BranchId;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

fn fork(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(name.into()), basis)
        .unwrap();
}
