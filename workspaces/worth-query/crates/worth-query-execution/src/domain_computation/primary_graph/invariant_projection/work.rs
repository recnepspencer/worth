#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionWork {
    equality_lookups: usize,
    index_candidates_examined: usize,
    adjacency_lists_read: usize,
    adjacency_edges_inspected: usize,
    endpoint_records_read: usize,
    field_reads: usize,
    aggregate_lookups: usize,
    aggregate_cache_hits: usize,
    aggregate_rebuild_input_rows: usize,
    reconstructive_scans: usize,
    output_lineage_source_selections: usize,
    output_lineage_role_lookups: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryInvariantProjectionWorkBudget {
    remaining: Option<usize>,
    exceeded: bool,
}

impl WorthQueryInvariantProjectionWork {
    pub const fn equality_lookups(self) -> usize {
        self.equality_lookups
    }

    pub const fn index_candidates_examined(self) -> usize {
        self.index_candidates_examined
    }

    pub const fn adjacency_lists_read(self) -> usize {
        self.adjacency_lists_read
    }

    pub const fn adjacency_edges_inspected(self) -> usize {
        self.adjacency_edges_inspected
    }

    pub const fn endpoint_records_read(self) -> usize {
        self.endpoint_records_read
    }

    pub const fn field_reads(self) -> usize {
        self.field_reads
    }

    pub const fn reconstructive_scans(self) -> usize {
        self.reconstructive_scans
    }

    pub const fn aggregate_lookups(self) -> usize {
        self.aggregate_lookups
    }

    pub const fn aggregate_cache_hits(self) -> usize {
        self.aggregate_cache_hits
    }

    pub const fn aggregate_rebuild_input_rows(self) -> usize {
        self.aggregate_rebuild_input_rows
    }

    pub const fn output_lineage_source_selections(self) -> usize {
        self.output_lineage_source_selections
    }

    pub const fn output_lineage_role_lookups(self) -> usize {
        self.output_lineage_role_lookups
    }

    pub const fn provider_work_units(self) -> usize {
        self.equality_lookups
            + self.index_candidates_examined
            + self.adjacency_lists_read
            + self.adjacency_edges_inspected
            + self.endpoint_records_read
            + self.field_reads
            + self.aggregate_lookups
            + self.reconstructive_scans
            + self.output_lineage_source_selections
            + self.output_lineage_role_lookups
    }

    pub(super) fn record_lookup(&mut self, examined: usize) {
        self.equality_lookups += 1;
        self.index_candidates_examined += examined;
    }

    pub(super) fn record_adjacency(&mut self, examined: usize, endpoints: usize) {
        self.adjacency_lists_read += 1;
        self.adjacency_edges_inspected += examined;
        self.endpoint_records_read += endpoints;
    }

    pub(super) fn record_field(&mut self) {
        self.field_reads += 1;
    }

    pub(super) fn record_aggregate_lookup(&mut self, cache_hit: bool, rebuild_rows: usize) {
        self.aggregate_lookups += 1;
        self.aggregate_cache_hits += usize::from(cache_hit);
        self.aggregate_rebuild_input_rows += rebuild_rows;
    }

    pub(super) fn record_output_lineage_selection(&mut self, lookups: usize) {
        self.output_lineage_source_selections += lookups;
    }

    pub(super) fn record_output_lineage_role_lookup(&mut self) {
        self.output_lineage_role_lookups += 1;
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    pub(super) const fn unbounded() -> Self {
        Self {
            remaining: None,
            exceeded: false,
        }
    }

    pub(super) const fn bounded(maximum: usize) -> Self {
        Self {
            remaining: Some(maximum),
            exceeded: false,
        }
    }

    pub(super) fn can_afford(&mut self, maximum_work: usize) -> bool {
        if self.exceeded {
            return false;
        }
        if self
            .remaining
            .is_some_and(|remaining| remaining < maximum_work)
        {
            self.exceeded = true;
            return false;
        }
        true
    }

    pub(super) fn consume(&mut self, actual_work: usize) {
        let Some(remaining) = self.remaining.as_mut() else {
            return;
        };
        *remaining = remaining
            .checked_sub(actual_work)
            .expect("provider work was preflighted before execution");
    }

    pub(super) fn mark_exceeded(&mut self) {
        self.exceeded = true;
    }

    pub(super) const fn remaining(&self) -> usize {
        match self.remaining {
            Some(remaining) => remaining,
            None => usize::MAX,
        }
    }

    pub(super) const fn exceeded(&self) -> bool {
        self.exceeded
    }
}
