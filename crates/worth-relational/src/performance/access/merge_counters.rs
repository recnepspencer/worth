use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_merge_history_ancestry_traversal(&self, nodes_visited: usize) {
        self.counters.count(|counters| {
            counters.merge_history_ancestry_traversals += 1;
            counters.merge_history_ancestry_nodes_visited += nodes_visited;
        });
    }

    pub(crate) fn count_merge_history_parent_comparisons(&self, comparisons: usize) {
        self.counters.count(|counters| {
            counters.merge_history_parent_comparisons += comparisons;
        });
    }

    pub(crate) fn count_merge_history_replay_planning(
        &self,
        nodes_visited: usize,
        parent_checks: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_history_replay_planning_nodes_visited += nodes_visited;
            counters.merge_history_replay_parent_checks += parent_checks;
        });
    }

    pub(crate) fn count_merge_history_durability_validation(
        &self,
        nodes_visited: usize,
        parent_checks: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_history_durability_validation_nodes_visited += nodes_visited;
            counters.merge_history_durability_parent_checks += parent_checks;
        });
    }
}
