use super::super::types::{DisjointApplyGroup, LoweredTask, ResolvedSignalPlannerPolicy};

pub(super) fn build_stage_apply_groups(
    tasks: &[LoweredTask],
    _policy: ResolvedSignalPlannerPolicy,
) -> Vec<DisjointApplyGroup> {
    tasks
        .iter()
        .enumerate()
        .map(|(task_index, _task)| DisjointApplyGroup {
            task_indices: vec![task_index],
        })
        .collect()
}
