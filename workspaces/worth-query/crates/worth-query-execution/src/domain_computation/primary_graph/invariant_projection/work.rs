/// Projection work and installed invariant read observations: equality lookups, index candidates,
/// adjacency and endpoint reads, field reads, aggregate lookups and cache hits,
/// and output-lineage lookups.
///
/// Observations for diagnostics and tests, not authority or a promised cost.
/// Installed invariant contexts include preparation and the current evaluation;
/// snapshot projections describe that projection alone. Unused read kinds are zero.
/// An installed rule counts acquired adjacency lists, inspected declared relation
/// records (including kinds filtered from its result), acquired entity metadata,
/// and field reads after acquiring the aspect state, even if decoding then fails.
/// Reads refused before acquisition add no observation. These are Query boundary
/// observations, not storage-engine I/O or authorization work.
///
/// Snapshot endpoint counts record metadata acquired by the endpoint reader,
/// including metadata rejected after acquisition. Endpoint reservation capacity
/// belongs to the private projection budget and is not an endpoint observation.
/// Snapshot field counts still include admitted calls that may stop before
/// acquiring a value; Phase 1 field-observation unification remains outstanding.
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
    overflowed: bool,
    consumed: usize,
    /// Start and maximum preflight demand of the current owner call.
    call: Option<(usize, usize)>,
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

    pub(in crate::domain_computation::primary_graph) fn record_adjacency(
        &mut self,
        examined: usize,
    ) {
        self.adjacency_lists_read += 1;
        self.adjacency_edges_inspected += examined;
    }

    pub(in crate::domain_computation::primary_graph) fn record_edge(&mut self) {
        self.adjacency_edges_inspected += 1;
    }

    pub(in crate::domain_computation::primary_graph) fn record_field(&mut self) {
        self.field_reads += 1;
    }
    pub(in crate::domain_computation::primary_graph) fn record_endpoint(&mut self) {
        self.endpoint_records_read += 1;
    }

    pub(super) fn record_aggregate_lookup(&mut self, cache_hit: bool, rebuild_rows: usize) {
        self.aggregate_lookups += 1;
        self.aggregate_cache_hits += usize::from(cache_hit);
        self.aggregate_rebuild_input_rows += rebuild_rows;
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
            overflowed: false,
            consumed: 0,
            call: None,
        }
    }

    pub(super) const fn bounded(maximum: usize) -> Self {
        Self {
            remaining: Some(maximum),
            exceeded: false,
            overflowed: false,
            consumed: 0,
            call: None,
        }
    }

    pub(super) fn can_afford(&mut self, maximum_work: usize) -> bool {
        if self.exceeded {
            return false;
        }
        if self.consumed.checked_add(maximum_work).is_none() {
            self.overflowed = true;
            self.exceeded = true;
            return false;
        }
        if self
            .remaining
            .is_some_and(|remaining| remaining < maximum_work)
        {
            self.exceeded = true;
            return false;
        }
        self.record_call_demand(maximum_work);
        true
    }

    #[must_use]
    pub(super) fn consume(&mut self, actual_work: usize) -> bool {
        if !self.can_afford(actual_work) {
            return false;
        }
        self.consumed += actual_work;
        if let Some(remaining) = self.remaining.as_mut() {
            *remaining = remaining
                .checked_sub(actual_work)
                .expect("provider work was preflighted before execution");
        }
        true
    }

    fn record_call_demand(&mut self, maximum_work: usize) {
        if let Some((start, demand)) = &mut self.call {
            let needed = self
                .consumed
                .checked_sub(*start)
                .and_then(|spent| spent.checked_add(maximum_work));
            if let Some(needed) = needed {
                *demand = (*demand).max(needed);
            } else {
                // The call cannot be carried when its charge is unrepresentable.
                self.call = None;
            }
        }
    }

    /// Rechecks need the adjacency read's existing charge plus a final admission
    /// check. A read refused before charging retains the empty-read allowance.
    /// Public diagnostic observations do not determine this private demand.
    pub(super) fn adjacency_recomparison_limit_since(self, before: Self) -> usize {
        self.consumed
            .saturating_sub(before.consumed)
            .saturating_add(1)
            .max(2)
    }

    pub(super) fn begin_call(&mut self) {
        self.call = Some((self.consumed, 0));
    }

    /// Required finite allowance and actual charge, independent of whether
    /// this execution selected a work ceiling.
    pub(super) fn end_call(&mut self) -> Option<(usize, usize)> {
        let (start, demand) = self.call.take()?;
        Some((demand, self.consumed.checked_sub(start)?))
    }

    /// Charges a call an earlier run made over the same facts, as if it were
    /// made now: it passes exactly when `demand` remains, and then consumes
    /// what it consumed through the budget's one checked spend. False,
    /// charging nothing, when it would not pass.
    pub(super) fn carry(&mut self, demand: usize, consumed: usize) -> bool {
        if self.exceeded
            || self.remaining.is_some_and(|remaining| remaining < demand)
            || consumed > demand
        {
            return false;
        }
        if self.consumed.checked_add(demand).is_none() {
            self.overflowed = true;
            self.exceeded = true;
            return false;
        }
        self.consume(consumed)
    }

    pub(super) const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub(super) fn mark_exceeded(&mut self) {
        self.exceeded = true;
    }

    pub(super) const fn remaining(&self) -> Option<usize> {
        self.remaining
    }

    pub(super) const fn exceeded(&self) -> bool {
        self.exceeded
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryInvariantProjectionWorkBudget as Budget;

    #[test]
    fn absent_projection_work_keeps_unrepresentable_charges_owned_and_refused() {
        let mut work = Budget::unbounded();
        let before = work;
        assert!(work.consume(usize::MAX));
        assert_eq!(work.adjacency_recomparison_limit_since(before), usize::MAX);
        let before_stop = work;
        assert!(!work.consume(1));
        assert_eq!(work.adjacency_recomparison_limit_since(before_stop), 2);
        assert_eq!(work.consumed, usize::MAX);
        assert_eq!(work.remaining(), None);
        assert!(work.exceeded());
        assert!(work.overflowed());
        let mut carried = Budget::unbounded();
        assert!(carried.consume(usize::MAX));
        assert!(!carried.carry(1, 1));
        assert!(carried.overflowed());
        assert_eq!(carried.consumed, usize::MAX);
    }
}
