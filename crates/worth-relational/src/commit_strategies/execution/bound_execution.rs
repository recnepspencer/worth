use std::sync::Arc;

use crate::capabilities::{AspectPlanSource, SchemaSource};
use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CommitStrategyExecutor, StrategyObservationContext,
};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

use super::error::StrategyExecutionError;
use super::read_contract_admission::validate_supported_read_contract;

pub(crate) struct BoundStrategyExecution<'runtime> {
    pub(super) request: &'runtime CanonicalStrategyCommitRequest,
    pub(super) executor: Arc<dyn CommitStrategyExecutor>,
    pub(super) observation: StrategyObservationContext<'runtime>,
}

pub(crate) fn bind_execution<'runtime>(
    runtime: &'runtime RelationalRuntime,
    request: &'runtime CanonicalStrategyCommitRequest,
    snapshot: &'runtime SnapshotHandle,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BoundStrategyExecution<'runtime>, StrategyExecutionError> {
    crate::visibility::materialization::read_records::read_request::consult(execution)
        .map_err(StrategyExecutionError::Execution)?;
    let descriptor = runtime
        .commit_strategy_registry()
        .get_by_id(request.strategy_id())
        .map(|registration| registration.descriptor())
        .ok_or(StrategyExecutionError::UnknownStrategyId {
            strategy_id: request.strategy_id(),
        })?;
    let executor_binding = runtime
        .commit_strategy_executor_registry()
        .get(request.strategy_id())
        .ok_or(StrategyExecutionError::UnboundStrategyExecutor {
            strategy_id: request.strategy_id(),
        })?;

    reject_descriptor_digest_mismatch(request, executor_binding.descriptor_digest)?;
    validate_supported_read_contract(request.strategy_id(), descriptor.read_contract())?;
    reject_foreign_runtime_snapshot(runtime, snapshot)?;

    let visibility = runtime.read_truth().project_snapshot(snapshot).ok_or(
        StrategyExecutionError::UnknownSnapshot {
            snapshot_id: snapshot.snapshot_id,
        },
    )?;

    Ok(BoundStrategyExecution {
        request,
        executor: Arc::clone(&executor_binding.executor),
        observation: StrategyObservationContext::new(
            runtime,
            snapshot,
            descriptor.read_contract(),
            runtime.schema_registry(),
            runtime.aspect_plan_catalog(),
            visibility,
        ),
    })
}

fn reject_descriptor_digest_mismatch(
    request: &CanonicalStrategyCommitRequest,
    bound_digest: crate::commit_strategies::data::CommitStrategyDescriptorDigest,
) -> Result<(), StrategyExecutionError> {
    if bound_digest != request.descriptor_digest() {
        return Err(StrategyExecutionError::DescriptorDigestMismatch {
            strategy_id: request.strategy_id(),
            request_digest: request.descriptor_digest(),
            bound_digest,
        });
    }
    Ok(())
}

fn reject_foreign_runtime_snapshot(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<(), StrategyExecutionError> {
    if snapshot.runtime_instance_id != runtime.runtime_instance_id() {
        return Err(StrategyExecutionError::UnknownSnapshot {
            snapshot_id: snapshot.snapshot_id,
        });
    }
    Ok(())
}
