use super::InspectionAccess;

impl<'runtime> InspectionAccess<'runtime> {
    pub(crate) fn count_graph_summary_request(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.inspection_graph_summary_requests += 1);
    }

    pub(crate) fn count_connectivity_summary_request(&self) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.inspection_connectivity_summary_requests += 1;
        });
    }

    pub(crate) fn count_neighbor_request(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.inspection_neighbor_requests += 1);
    }

    pub(crate) fn count_commit_read(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.inspection_commit_reads += 1);
    }

    pub(crate) fn count_historical_view_open(&self) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.inspection_historical_view_opens += 1;
        });
    }

    pub(crate) fn count_budget_refusal(&self) {
        self.runtime
            .performance_access()
            .count_inspection_budget_refusal();
    }

    pub(crate) fn count_connectivity_work(
        &self,
        entity_scans: u64,
        relation_scans: u64,
        frontier_expansions: u64,
        components_evaluated: u64,
    ) {
        self.runtime
            .performance_access()
            .count_inspection_connectivity_work(
                entity_scans,
                relation_scans,
                frontier_expansions,
                components_evaluated,
            );
    }

    pub(crate) fn count_retention_work(&self, entity_slot_scans: u64, relation_slot_scans: u64) {
        self.runtime
            .performance_access()
            .count_inspection_retention_work(entity_slot_scans, relation_slot_scans);
    }
}
