use crate::data::error::SignalError;
use crate::logic::transaction::runtime::{
    BranchMergeExecutionSummary, BranchMergePlan, BranchMergeResult,
    LoweredFoundationalMergeRequest,
};
use crate::state::SignalSnapshotId;

use super::super::super::runtime_state::SignalRuntime;
use super::execution_application;
use super::execution_artifacts;
use super::execution_finalization;
use super::execution_preparation;
use super::execution_summary;
use super::result_projection;

pub(super) fn execute_and_project<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    request: &LoweredFoundationalMergeRequest,
    plan: &BranchMergePlan,
) -> Result<BranchMergeResult, SignalError>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime
        .branches
        .ensure_snapshot_storage_available()
        .map_err(|denial| {
            SignalError::invalid_input(format!(
                "Signal merge snapshot storage exhausted before movement: {denial:?}"
            ))
        })?;
    let raw_request = request.normalized_request().request();
    let next_target_generation = runtime
        .branches
        .next_branch_head_generation(raw_request.target_branch.id)
        .map_err(|denial| {
            SignalError::internal(format!(
                "Signal merge target generation cannot advance: {denial:?}"
            ))
        })?;
    let snapshot_id = reserve_target_snapshot_identity(runtime, raw_request.target_branch.id)?;
    runtime
        .branches
        .mark_merge_participants(raw_request.source_branch.id, raw_request.target_branch.id);
    let execution = execute_plan(runtime, request, plan, snapshot_id);
    runtime
        .branches
        .clear_merge_participants(raw_request.source_branch.id, raw_request.target_branch.id);
    let summary = match execution {
        Ok(summary) => summary,
        Err(error) => {
            crate::diagnostics::recorder::record_branch_merge_failure(
                &mut runtime.graph,
                &error,
                Some(raw_request.source_branch.clone()),
                Some(raw_request.target_branch.clone()),
            );
            return Err(error);
        }
    };
    runtime
        .branches
        .commit_branch_head_generation(raw_request.target_branch.id, next_target_generation);
    crate::diagnostics::recorder::record_branch_merge_summary(
        &mut runtime.graph,
        &summary,
        raw_request.source_branch.name.clone(),
        raw_request.target_branch.name.clone(),
    );
    Ok(result_projection::project(summary))
}

#[cfg(test)]
pub(super) fn execute_summary_for_test<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    request: &LoweredFoundationalMergeRequest,
    plan: &BranchMergePlan,
) -> Result<BranchMergeExecutionSummary, SignalError>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let target_branch_id = request.normalized_request().request().target_branch.id;
    let snapshot_id = reserve_target_snapshot_identity(runtime, target_branch_id)?;
    execute_plan(runtime, request, plan, snapshot_id)
}

fn execute_plan<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    request: &LoweredFoundationalMergeRequest,
    plan: &BranchMergePlan,
    snapshot_id: SignalSnapshotId,
) -> Result<BranchMergeExecutionSummary, SignalError>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let raw_request = request.normalized_request().request();
    let mut prepared = execution_preparation::prepare(runtime, raw_request, plan)?;
    execution_application::apply_governed_merge(&mut prepared, plan)?;
    let artifacts =
        execution_artifacts::finalize_artifacts(&mut prepared, raw_request, plan, snapshot_id)?;
    let finalization = execution_finalization::finalize_branch_state(
        runtime,
        raw_request,
        plan,
        prepared,
        artifacts,
    )?;
    Ok(execution_summary::build_execution_summary(
        plan,
        finalization,
    ))
}

fn reserve_target_snapshot_identity<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    target_branch_id: crate::state::SignalBranchId,
) -> Result<SignalSnapshotId, SignalError>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let next_snapshot_id = runtime
        .branches
        .replay_graph(
            target_branch_id,
            runtime.graph.current_branch().id,
            &runtime.graph,
        )
        .ok_or_else(|| SignalError::unknown_branch(Some(target_branch_id), "merge-target"))?
        .diagnostics_state()
        .branch_snapshot_allocator_state()
        .0;
    runtime
        .branches
        .synchronize_snapshot_identity_high_water(next_snapshot_id);
    runtime
        .branches
        .reserve_snapshot_identity()
        .map_err(|snapshot_id| {
            SignalError::invalid_input(format!(
                "Signal merge snapshot identity exhausted at {} before movement",
                snapshot_id.0
            ))
        })
}
