use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, LoweredStrategyCommitPlan, NativeStrategyCommitRequest,
    StrategyCommitRequestError, StrategyExecutionDraft, StrategyLoweringError,
};
use crate::commit_strategies::{
    bind_execution, canonicalize_request, execute_bound_strategy, lower_execution,
    validate_lowered_plan as validate_lowered_strategy_plan,
};
use crate::mvcc::RelationalTransactionValidationInput;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;
use crate::transactions::data::TransactionCommitError;

#[derive(Debug, Clone, Copy)]
pub struct CommitStrategiesFacade<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> CommitStrategiesFacade<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn canonicalize_request(
        &self,
        request: &NativeStrategyCommitRequest,
    ) -> Result<CanonicalStrategyCommitRequest, StrategyCommitRequestError> {
        canonicalize_request(self.runtime.commit_strategy_registry(), request)
    }

    pub fn execute(
        &self,
        request: &CanonicalStrategyCommitRequest,
        snapshot: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<StrategyExecutionDraft, crate::commit_strategies::StrategyExecutionError> {
        let bound = bind_execution(self.runtime, request, snapshot, execution)?;
        execute_bound_strategy(execution, bound)
    }
}

/// Stateless owner port. Each governed transition borrows the runtime only for
/// that transition; lowered and validated artifacts remain detached between calls.
#[derive(Debug, Default)]
pub struct CommitStrategiesAuthorityFacade;

impl CommitStrategiesAuthorityFacade {
    pub(crate) fn new() -> Self {
        Self
    }

    pub fn lower_execution(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        request: &CanonicalStrategyCommitRequest,
        execution: &StrategyExecutionDraft,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        intent: crate::mvcc::RelationalTransactionIntent,
    ) -> Result<LoweredStrategyCommitPlan, StrategyLoweringError> {
        let transaction = runtime
            .begin_branch_transaction(basis, intent)
            .map_err(crate::commit_strategies::lowering::strategy_transaction_admission_error)?;
        lower_execution(request_execution, runtime, request, execution, transaction)
    }

    pub(crate) fn lower_execution_with_input(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        request: &CanonicalStrategyCommitRequest,
        execution: &StrategyExecutionDraft,
        input: RelationalTransactionValidationInput,
    ) -> Result<LoweredStrategyCommitPlan, StrategyLoweringError> {
        let transaction = runtime
            .begin_branch_transaction_with_owner_inputs(input)
            .map_err(crate::commit_strategies::lowering::strategy_transaction_admission_error)?;
        lower_execution(request_execution, runtime, request, execution, transaction)
    }

    pub fn validate_lowered_plan(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        lowered: LoweredStrategyCommitPlan,
    ) -> Result<crate::mvcc::ValidatedRelationalProposal, TransactionCommitError> {
        validate_lowered_strategy_plan(execution, runtime, lowered)
    }

    pub fn prepare_validated_commit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        validated: crate::mvcc::ValidatedRelationalProposal,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        runtime.prepare_validated_proposal(execution, validated)
    }

    #[cfg(test)]
    pub(crate) fn execute_validated_commit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        validated: crate::mvcc::ValidatedRelationalProposal,
    ) -> Result<crate::transactions::data::CommitResult, TransactionCommitError> {
        let candidate = self.prepare_validated_commit(execution, runtime, validated)?;
        runtime.publish_prepared_candidate(execution, candidate)
    }
}

#[cfg(test)]
#[path = "facade_tests/mod.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::native_strategy_fixtures::{
    execute_persisted_intent_strategy_commit, lowered_commit_fixture, persisted_intent_runtime,
};
