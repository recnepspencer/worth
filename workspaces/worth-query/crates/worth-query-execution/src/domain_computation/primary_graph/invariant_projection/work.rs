/// Work one invariant projection spent: equality lookups, index candidates,
/// adjacency and endpoint reads, field reads, aggregate lookups and cache hits,
/// and output-lineage lookups.
///
/// Descriptive evidence for budgeting; it grants nothing.
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
    output_lineage_source_selections: usize,
    output_lineage_role_lookups: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryInvariantProjectionWorkBudget {
    remaining: Option<usize>,
    exceeded: bool,
    /// While one owner call of a partitioned computation holds the reader:
    /// the least remaining work any of its preflights or charges left.
    call_floor: Option<usize>,
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
            + self.output_lineage_source_selections
            + self.output_lineage_role_lookups
    }

    /// The work done since `before`, an earlier reading of this same
    /// projection. `None` when `before` is not earlier.
    pub(super) fn since(self, before: Self) -> Option<Self> {
        self.each(before, usize::checked_sub)
    }

    /// This work with `carried` done again: the work a call did on an earlier
    /// run. `None` when a count overflows.
    pub(super) fn with_carried(self, carried: Self) -> Option<Self> {
        self.each(carried, usize::checked_add)
    }

    fn each(self, other: Self, op: fn(usize, usize) -> Option<usize>) -> Option<Self> {
        Some(Self {
            equality_lookups: op(self.equality_lookups, other.equality_lookups)?,
            index_candidates_examined: op(
                self.index_candidates_examined,
                other.index_candidates_examined,
            )?,
            adjacency_lists_read: op(self.adjacency_lists_read, other.adjacency_lists_read)?,
            adjacency_edges_inspected: op(
                self.adjacency_edges_inspected,
                other.adjacency_edges_inspected,
            )?,
            endpoint_records_read: op(self.endpoint_records_read, other.endpoint_records_read)?,
            field_reads: op(self.field_reads, other.field_reads)?,
            aggregate_lookups: op(self.aggregate_lookups, other.aggregate_lookups)?,
            aggregate_cache_hits: op(self.aggregate_cache_hits, other.aggregate_cache_hits)?,
            aggregate_rebuild_input_rows: op(
                self.aggregate_rebuild_input_rows,
                other.aggregate_rebuild_input_rows,
            )?,
            output_lineage_source_selections: op(
                self.output_lineage_source_selections,
                other.output_lineage_source_selections,
            )?,
            output_lineage_role_lookups: op(
                self.output_lineage_role_lookups,
                other.output_lineage_role_lookups,
            )?,
        })
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
            call_floor: None,
        }
    }

    pub(super) const fn bounded(maximum: usize) -> Self {
        Self {
            remaining: Some(maximum),
            exceeded: false,
            call_floor: None,
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
        // Either the remaining work covers the preflight, or nothing bounds it.
        if let Some(left) = self.remaining().checked_sub(maximum_work) {
            self.lower_call_floor(left);
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
        let left = *remaining;
        self.lower_call_floor(left);
    }

    fn lower_call_floor(&mut self, left: usize) {
        if let Some(floor) = &mut self.call_floor {
            *floor = (*floor).min(left);
        }
    }

    /// Begins one owner call: from here the budget remembers how low its
    /// preflights and charges took the remaining work.
    pub(super) fn begin_call(&mut self) {
        self.call_floor = Some(self.remaining());
    }

    /// Ends the call begun last, when it started with `remaining_at_start`.
    /// Returns the work it needed remaining at its start to pass every
    /// preflight, and the work it consumed.
    pub(super) fn end_call(&mut self, remaining_at_start: usize) -> Option<(usize, usize)> {
        let floor = self.call_floor.take()?;
        Some((
            remaining_at_start.checked_sub(floor)?,
            remaining_at_start.checked_sub(self.remaining())?,
        ))
    }

    /// Charges a call an earlier run made over the same facts, as if it were
    /// made now: it passes exactly when `demand` remains, and then consumes
    /// what it consumed through the budget's one checked spend. False,
    /// charging nothing, when it would not pass.
    pub(super) fn carry(&mut self, demand: usize, consumed: usize) -> bool {
        if self.exceeded || self.remaining() < demand || consumed > demand {
            return false;
        }
        self.consume(consumed);
        true
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
