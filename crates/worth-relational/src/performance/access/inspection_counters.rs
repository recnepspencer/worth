use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_inspection_connectivity_work(
        &self,
        entity_scans: u64,
        relation_scans: u64,
        frontier_expansions: u64,
        components_evaluated: u64,
    ) {
        let entity_scans = entity_scans.min(usize::MAX as u64) as usize;
        let relation_scans = relation_scans.min(usize::MAX as u64) as usize;
        let frontier_expansions = frontier_expansions.min(usize::MAX as u64) as usize;
        let components_evaluated = components_evaluated.min(usize::MAX as u64) as usize;
        self.counters.count(|counters| {
            counters.inspection_connectivity_entity_scans += entity_scans;
            counters.inspection_connectivity_relation_scans += relation_scans;
            counters.inspection_connectivity_frontier_expansions += frontier_expansions;
            counters.inspection_connectivity_components_evaluated += components_evaluated;
        });
    }

    pub(crate) fn count_inspection_retention_work(
        &self,
        entity_slot_scans: u64,
        relation_slot_scans: u64,
    ) {
        let entity_slot_scans = entity_slot_scans.min(usize::MAX as u64) as usize;
        let relation_slot_scans = relation_slot_scans.min(usize::MAX as u64) as usize;
        self.counters.count(|counters| {
            counters.inspection_retention_entity_slot_scans += entity_slot_scans;
            counters.inspection_retention_relation_slot_scans += relation_slot_scans;
        });
    }

    pub(crate) fn count_inspection_budget_refusal(&self) {
        self.counters.count(|counters| {
            counters.inspection_budget_refusals += 1;
        });
    }
}
