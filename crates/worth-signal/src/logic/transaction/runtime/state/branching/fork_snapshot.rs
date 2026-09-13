use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::summary::{ExecutionHistorySummary, GraphSummary};
use crate::logic::transaction::runtime::state::branching::SignalBranchForkDenial;
use crate::state::{
    SignalBranchHandle, SignalSnapshotV1, SnapshotDependencyRestoreMode, SnapshotRestoreIntent,
};

use super::super::runtime_state::SignalRuntime;
use super::branches::BranchState;

pub(super) fn materialize_snapshot_fork_state<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    parent_branch: SignalBranchHandle,
    snapshot: &SignalSnapshotV1,
) -> Result<BranchState<D, I, T>, SignalBranchForkDenial>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    let reconstructability_proof = snapshot.reconstructability_proof().map_err(|_| {
        SignalBranchForkDenial::UnknownForkSnapshot {
            parent_branch_id: parent_branch.id,
            snapshot_id: snapshot.meta.snapshot_id,
        }
    })?;
    let restore_plan = runtime
        .graph
        .plan_snapshot_restore(snapshot, SnapshotRestoreIntent::restore_runtime_truth())
        .map_err(|_| SignalBranchForkDenial::UnknownForkSnapshot {
            parent_branch_id: parent_branch.id,
            snapshot_id: snapshot.meta.snapshot_id,
        })?;
    if matches!(
        SnapshotRestoreIntent::restore_runtime_truth().dependency_state,
        SnapshotDependencyRestoreMode::SeedRecomputationFromSnapshot
    ) {
        unreachable!("runtime truth restore intent does not request seed recomputation");
    }
    let snapshot_state = runtime
        .branches
        .snapshot_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id)
        .ok_or(SignalBranchForkDenial::UnknownForkSnapshot {
            parent_branch_id: parent_branch.id,
            snapshot_id: snapshot.meta.snapshot_id,
        })?;
    SignalRuntime::<D, I, E, Ctx, T>::ensure_managed_queue_branch_transfer_allowed(
        snapshot_state.resource(),
    )
    .map_err(SignalRuntime::<D, I, E, Ctx, T>::branch_transfer_error_to_fork_denial)?;
    let snapshot_state = snapshot_state.clone();
    let mut graph =
        snapshot
            .authority_graph()
            .map_err(|_| SignalBranchForkDenial::UnknownForkSnapshot {
                parent_branch_id: parent_branch.id,
                snapshot_id: snapshot.meta.snapshot_id,
            })?;
    if let Some(mut telemetry) = graph.telemetry_mut() {
        *telemetry = snapshot.checkpoint_image.graph_telemetry;
    }
    for requirement in &reconstructability_proof.required_rebuild {
        match requirement {
            crate::logic::transaction::RequiredDerivedRebuildSet::DependencyIndexes(_) => {
                graph
                    .apply_classified_snapshot_batch_commit(
                        restore_plan.checkpoint_restore_batch().clone_inner(),
                    )
                    .map_err(|_| SignalBranchForkDenial::UnknownForkSnapshot {
                        parent_branch_id: parent_branch.id,
                        snapshot_id: snapshot.meta.snapshot_id,
                    })?;
            }
            crate::logic::transaction::RequiredDerivedRebuildSet::ReplaySuffix(replay) => {
                if snapshot.diagnostics.replay_frames.len() < replay.replay_event_count as usize {
                    return Err(SignalBranchForkDenial::UnknownForkSnapshot {
                        parent_branch_id: parent_branch.id,
                        snapshot_id: snapshot.meta.snapshot_id,
                    });
                }
            }
            crate::logic::transaction::RequiredDerivedRebuildSet::MergeSupport(_) => {
                graph.clear_branch_mutation_nodes();
            }
            crate::logic::transaction::RequiredDerivedRebuildSet::TemporalState(_) => {}
        }
    }
    graph
        .readmit_checkpoint_causes()
        .map_err(|_| SignalBranchForkDenial::UnknownForkSnapshot {
            parent_branch_id: parent_branch.id,
            snapshot_id: snapshot.meta.snapshot_id,
        })?;
    graph
        .diagnostics_state_mut()
        .restore_snapshot_payload(snapshot.diagnostics.clone());
    graph
        .diagnostics_state_mut()
        .set_active_branch(parent_branch.id);
    graph
        .diagnostics_state_mut()
        .stage_branch_head_snapshot_projection(parent_branch.id, snapshot.meta.snapshot_id);
    let mut state = snapshot_state.into_branch_state(graph, snapshot.runtime_telemetry);
    let retention_budget = state.graph().installed_runtime_policy().retention_budget();
    let profile = state.graph().diagnostics_profile();
    let history = ExecutionHistorySummary::from_graph(
        state.graph(),
        profile,
        retention_budget.detail_limit,
        retention_budget.retain_history_details,
        OrdinaryAccessLane,
    );
    let graph_summary = GraphSummary::from_graph(
        state.graph(),
        profile,
        retention_budget.detail_limit,
        OrdinaryAccessLane,
    );
    state
        .graph_mut()
        .diagnostics_state_mut()
        .refresh_retained_views(history, graph_summary);
    Ok(state)
}
