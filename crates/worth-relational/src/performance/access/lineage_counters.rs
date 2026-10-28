use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    #[cfg(test)]
    pub(crate) fn count_lineage_graph_snapshot_request(
        &self,
        node_count: usize,
        event_count: usize,
    ) {
        self.counters.count(|counters| {
            counters.lineage_graph_snapshot_requests += 1;
            counters.lineage_graph_snapshot_nodes_materialized += node_count;
            counters.lineage_graph_snapshot_events_materialized += event_count;
        });
    }

    pub(crate) fn count_lineage_finalization(
        &self,
        event_batch_width: usize,
        decision_log_width: usize,
    ) {
        self.counters.count(|counters| {
            counters.lineage_finalization_event_batch_width += event_batch_width;
            counters.lineage_finalization_decision_log_width += decision_log_width;
        });
    }

    pub(crate) fn count_lineage_publication_artifact(
        &self,
        event_width: usize,
        decision_width: usize,
    ) {
        self.counters.count(|counters| {
            counters.lineage_publication_event_width += event_width;
            counters.lineage_publication_decision_width += decision_width;
        });
    }

    #[cfg(test)]
    pub(crate) fn count_lineage_graph_snapshot_visibility_cache(&self, hit: bool) {
        self.counters.count(|counters| {
            if hit {
                counters.lineage_graph_snapshot_visibility_cache_hits += 1;
            } else {
                counters.lineage_graph_snapshot_visibility_cache_miss_reconstructions += 1;
            }
        });
    }

    pub(crate) fn count_lineage_historical_resolution(
        &self,
        index_probes: usize,
        event_visits: usize,
        traversed_events: usize,
        reachable_commit_node_visits: usize,
        reachable_commit_parent_edge_visits: usize,
        reachable_commit_catalog_probes: usize,
    ) {
        self.counters.count(|counters| {
            counters.lineage_historical_resolution_requests += 1;
            counters.lineage_historical_resolution_index_probes += index_probes;
            counters.lineage_historical_resolution_event_visits += event_visits;
            counters.lineage_historical_resolution_traversed_events += traversed_events;
            counters.lineage_historical_resolution_reachable_commit_node_visits +=
                reachable_commit_node_visits;
            counters.lineage_historical_resolution_reachable_commit_parent_edge_visits +=
                reachable_commit_parent_edge_visits;
            counters.lineage_historical_resolution_reachable_commit_catalog_probes +=
                reachable_commit_catalog_probes;
        });
    }

    #[cfg(test)]
    pub(crate) fn count_lineage_branch_divergence(
        &self,
        left_event_count: usize,
        right_event_count: usize,
        left_node_count: usize,
        right_node_count: usize,
    ) {
        self.counters.count(|counters| {
            counters.lineage_branch_divergence_requests += 1;
            counters.lineage_branch_divergence_event_scans += left_event_count + right_event_count;
            counters.lineage_branch_divergence_node_scans += left_node_count + right_node_count;
        });
    }
}
