//! Crate-boundary internal port for `WorthQueryApplicationCleanupRequired`.
//! Never exported on `worth-query-execution::facade` or Host.

use super::super::{RetainedOriginalCleanupCommit, WorthQueryApplicationCommitOutcome};
use super::WorthQueryApplicationCleanupRequired;
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryProductRuntime, WorthQueryMutationCleanupFailure,
};

pub trait ApplicationCleanupRequiredInternalPort: Sized {
    fn reserve(
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Box<Self>, worth_execution::ExecutionAllocationDenial>;

    fn fill(
        self: Box<Self>,
        outcome: WorthQueryApplicationCommitOutcome,
        failure: WorthQueryMutationCleanupFailure,
        runtime: WorthQueryProductRuntime,
        original: Option<RetainedOriginalCleanupCommit>,
        bridge: Box<dyn std::any::Any + Send + Sync>,
    ) -> Box<Self>;
}

impl ApplicationCleanupRequiredInternalPort for WorthQueryApplicationCleanupRequired {
    fn reserve(
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Box<Self>, worth_execution::ExecutionAllocationDenial> {
        Self::reserve(policy)
    }

    fn fill(
        self: Box<Self>,
        outcome: WorthQueryApplicationCommitOutcome,
        failure: WorthQueryMutationCleanupFailure,
        runtime: WorthQueryProductRuntime,
        original: Option<RetainedOriginalCleanupCommit>,
        bridge: Box<dyn std::any::Any + Send + Sync>,
    ) -> Box<Self> {
        Self::fill(self, outcome, failure, runtime, original, bridge)
    }
}
