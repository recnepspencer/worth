use serde::{Deserialize, Serialize};

use worth_foundational::{ExecutionObjectiveProfile, ExecutionPosture};

use crate::data::graph::SignalGraph;

/// Installed Signal thresholds resolved against one caller's execution authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSignalPlannerPolicy {
    posture: ExecutionPosture,
    objective: ExecutionObjectiveProfile,
    parallel_min_tasks: usize,
    full_parallel_min_tasks: usize,
    chunk_size: usize,
    apply_group_min_width: usize,
    max_concurrent_apply_groups: usize,
}

impl ResolvedSignalPlannerPolicy {
    pub(crate) fn for_graph(
        graph: &SignalGraph,
        lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Self {
        let installed = graph.installed_runtime_policy();
        let posture = lease.resolved_posture();
        let workers = if posture == ExecutionPosture::Automatic {
            lease.max_workers()
        } else {
            1
        };
        Self {
            posture,
            objective: installed.execution_objective(),
            parallel_min_tasks: installed.parallel_min_tasks().max(1),
            full_parallel_min_tasks: installed.full_parallel_min_tasks().max(1),
            chunk_size: 1,
            apply_group_min_width: 1,
            max_concurrent_apply_groups: workers,
        }
    }

    pub const fn posture(self) -> ExecutionPosture {
        self.posture
    }

    pub const fn objective(self) -> ExecutionObjectiveProfile {
        self.objective
    }

    pub const fn parallel_min_tasks(self) -> usize {
        self.parallel_min_tasks
    }

    pub const fn full_parallel_min_tasks(self) -> usize {
        self.full_parallel_min_tasks
    }

    pub fn chunk_size_for(self, task_count: usize) -> usize {
        self.chunk_size.min(task_count.max(1))
    }

    pub const fn apply_group_min_width(self) -> usize {
        self.apply_group_min_width
    }

    pub fn max_apply_group_count_for(self, task_count: usize) -> usize {
        self.max_concurrent_apply_groups.min(task_count.max(1))
    }
}
