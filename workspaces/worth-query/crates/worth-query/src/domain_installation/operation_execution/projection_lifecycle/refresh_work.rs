use crate::ordinary::live::WorthQueryManagedLiveDelivery;

/// Exact work performed by one lifecycle refresh.
///
/// Promotion counters deliberately do not represent this lane: refresh owns
/// an already-open resource and reports its maintenance, delivery, read,
/// projection, and native-rebind work separately.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryLiveProjectionRefreshWork {
    authority_checks: usize,
    drain_calls: usize,
    delivery_batches: usize,
    maintenance_batches: usize,
    mutation_deltas: usize,
    affected_requirement_rows: usize,
    touched_edges: usize,
    touched_frontiers: usize,
    index_updates: usize,
    live_view_updates: usize,
    skipped_unaffected_requirements: usize,
    strategy_recomputes: usize,
    background_index_builds: usize,
    read_calls: usize,
    projection_calls: usize,
    native_rebind_calls: usize,
    impact_classifications: usize,
}

impl WorthQueryLiveProjectionRefreshWork {
    pub(in crate::domain_installation::operation_execution) fn authority_checked() -> Self {
        Self {
            authority_checks: 1,
            ..Self::default()
        }
    }

    pub(super) fn retain_delivery(&mut self, delivery: &WorthQueryManagedLiveDelivery) {
        self.delivery_batches = delivery.batches().len();
        for batch in delivery.batches() {
            let Some(work) = batch.maintenance_work() else {
                continue;
            };
            self.maintenance_batches += 1;
            self.mutation_deltas += work.mutation_delta_count();
            self.affected_requirement_rows += work.affected_requirement_row_count();
            self.touched_edges += work.touched_edge_count();
            self.touched_frontiers += work.touched_frontier_count();
            self.index_updates += work.index_update_count();
            self.live_view_updates += work.live_view_update_count();
            self.skipped_unaffected_requirements += work.skipped_unaffected_requirement_count();
            self.strategy_recomputes += work.strategy_recompute_count();
            self.background_index_builds += work.background_index_build_count();
        }
    }

    pub(super) fn begin_drain(&mut self) {
        self.drain_calls = 1;
    }

    pub(super) fn begin_read(&mut self) {
        self.read_calls = 1;
    }

    pub(super) fn retain_impact(
        &mut self,
        _impact: &crate::domain_installation::WorthQueryImpactDecision,
    ) {
        self.impact_classifications = 1;
    }

    pub(super) fn retain_projection(&mut self) {
        self.projection_calls = 1;
    }

    pub(super) fn begin_native_rebind(&mut self) {
        self.native_rebind_calls = 1;
    }

    pub fn authority_checks(self) -> usize {
        self.authority_checks
    }

    pub fn drain_calls(self) -> usize {
        self.drain_calls
    }

    pub fn delivery_batches(self) -> usize {
        self.delivery_batches
    }

    pub fn maintenance_batches(self) -> usize {
        self.maintenance_batches
    }

    pub fn mutation_deltas(self) -> usize {
        self.mutation_deltas
    }

    pub fn affected_requirement_rows(self) -> usize {
        self.affected_requirement_rows
    }

    pub fn touched_edges(self) -> usize {
        self.touched_edges
    }

    pub fn touched_frontiers(self) -> usize {
        self.touched_frontiers
    }

    pub fn index_updates(self) -> usize {
        self.index_updates
    }

    pub fn live_view_updates(self) -> usize {
        self.live_view_updates
    }

    pub fn skipped_unaffected_requirements(self) -> usize {
        self.skipped_unaffected_requirements
    }

    pub fn strategy_recomputes(self) -> usize {
        self.strategy_recomputes
    }

    pub fn background_index_builds(self) -> usize {
        self.background_index_builds
    }

    pub fn read_calls(self) -> usize {
        self.read_calls
    }

    pub fn projection_calls(self) -> usize {
        self.projection_calls
    }

    pub fn native_rebind_calls(self) -> usize {
        self.native_rebind_calls
    }

    pub fn impact_classifications(self) -> usize {
        self.impact_classifications
    }
}
