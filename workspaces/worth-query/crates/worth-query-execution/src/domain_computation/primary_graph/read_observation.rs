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
/// Endpoint counts describe explicit metadata observation operations, including
/// result-tree nodes and metadata rejected after acquisition. Fetching a record
/// internally to read a field does not add an endpoint observation.
/// Endpoint reservation capacity
/// belongs to the private projection budget and is not an endpoint observation.
/// Field reads include acquired absent values and reads whose decoding fails.
/// Reuse contributes no historical reads; restarting a computation retains the
/// observations already made. Counts saturate at `usize::MAX` and never admit work.
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
            .saturating_add(self.index_candidates_examined)
            .saturating_add(self.adjacency_lists_read)
            .saturating_add(self.adjacency_edges_inspected)
            .saturating_add(self.endpoint_records_read)
            .saturating_add(self.field_reads)
            .saturating_add(self.aggregate_lookups)
            .saturating_add(self.output_lineage_source_selections)
            .saturating_add(self.output_lineage_role_lookups)
    }

    /// Combine independent read stages, without changing their resource policy.
    pub(super) fn combined(self, other: Self) -> Self {
        Self {
            equality_lookups: self.equality_lookups.saturating_add(other.equality_lookups),
            index_candidates_examined: self
                .index_candidates_examined
                .saturating_add(other.index_candidates_examined),
            adjacency_lists_read: self
                .adjacency_lists_read
                .saturating_add(other.adjacency_lists_read),
            adjacency_edges_inspected: self
                .adjacency_edges_inspected
                .saturating_add(other.adjacency_edges_inspected),
            endpoint_records_read: self
                .endpoint_records_read
                .saturating_add(other.endpoint_records_read),
            field_reads: self.field_reads.saturating_add(other.field_reads),
            aggregate_lookups: self
                .aggregate_lookups
                .saturating_add(other.aggregate_lookups),
            aggregate_cache_hits: self
                .aggregate_cache_hits
                .saturating_add(other.aggregate_cache_hits),
            aggregate_rebuild_input_rows: self
                .aggregate_rebuild_input_rows
                .saturating_add(other.aggregate_rebuild_input_rows),
            output_lineage_source_selections: self
                .output_lineage_source_selections
                .saturating_add(other.output_lineage_source_selections),
            output_lineage_role_lookups: self
                .output_lineage_role_lookups
                .saturating_add(other.output_lineage_role_lookups),
        }
    }

    pub(super) fn record_lookup(&mut self, examined: usize) {
        self.equality_lookups = self.equality_lookups.saturating_add(1);
        self.index_candidates_examined = self.index_candidates_examined.saturating_add(examined);
    }

    pub(super) fn record_index_candidates(&mut self, examined: usize) {
        self.index_candidates_examined = self.index_candidates_examined.saturating_add(examined);
    }

    pub(super) fn record_adjacencies(&mut self, lists: usize, examined: usize) {
        self.adjacency_lists_read = self.adjacency_lists_read.saturating_add(lists);
        self.adjacency_edges_inspected = self.adjacency_edges_inspected.saturating_add(examined);
    }

    pub(super) fn record_edge(&mut self) {
        self.adjacency_edges_inspected = self.adjacency_edges_inspected.saturating_add(1);
    }

    pub(super) fn record_fields(&mut self, reads: usize) {
        self.field_reads = self.field_reads.saturating_add(reads);
    }
    pub(super) fn record_endpoint(&mut self) {
        self.record_endpoints(1);
    }

    pub(super) fn record_endpoints(&mut self, reads: usize) {
        self.endpoint_records_read = self.endpoint_records_read.saturating_add(reads);
    }

    pub(super) fn record_aggregate_lookup(&mut self, cache_hit: bool, rebuild_rows: usize) {
        self.aggregate_lookups = self.aggregate_lookups.saturating_add(1);
        self.aggregate_cache_hits = self
            .aggregate_cache_hits
            .saturating_add(usize::from(cache_hit));
        self.aggregate_rebuild_input_rows = self
            .aggregate_rebuild_input_rows
            .saturating_add(rebuild_rows);
    }

    pub(super) fn record_output_lineage_role_lookup(&mut self) {
        self.output_lineage_role_lookups = self.output_lineage_role_lookups.saturating_add(1);
    }
}
