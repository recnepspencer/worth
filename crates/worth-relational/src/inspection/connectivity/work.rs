#[derive(Default)]
pub(super) struct ConnectivityWork {
    pub(super) entity_scans: u64,
    pub(super) relation_scans: u64,
    pub(super) frontier_expansions: u64,
    pub(super) components_evaluated: u64,
    pub(super) work_units: u64,
}

impl ConnectivityWork {
    pub(super) fn record_entity_scan(&mut self) {
        self.entity_scans += 1;
        self.work_units += 1;
    }

    pub(super) fn record_relation_scan(&mut self) {
        self.relation_scans += 1;
        self.work_units += 1;
    }

    pub(super) fn record_frontier_expansion(&mut self) {
        self.frontier_expansions += 1;
        self.work_units += 1;
    }

    pub(super) fn record_component(&mut self) {
        self.components_evaluated += 1;
        self.work_units += 1;
    }
}
