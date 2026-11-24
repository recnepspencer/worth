use crate::history::data::CommitId;
use crate::replay::data::ReplayFailureClass;
use crate::runtime::RelationalRuntime;

pub(super) fn replay_commit_closure_by_commit_id_order(
    runtime: &RelationalRuntime,
    commit_id: CommitId,
) -> Result<Vec<CommitId>, ReplayFailureClass> {
    let closure = crate::replay::planning::replay_commit_closure_by_commit_id_order(
        &runtime.history,
        commit_id,
    );
    runtime
        .performance_access()
        .count_merge_history_replay_planning(closure.nodes_visited, closure.parent_checks);
    closure.commits
}
