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
