/// Algorithmic contract used to qualify authored workflow structure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowValidationComplexityContract {
    IndexedSparseGraphWithLogarithmicDominance,
}

/// The separately metered cost of validating the provisional retry form.
///
/// Retry control flow becomes a fully qualified supported form in Phase 3. Until
/// then, each authored retry is checked by one allocation-reusing traversal of
/// the sparse control graph rather than being included in the ordinary linear
/// authoring claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowRetryValidationComplexityContract {
    PerRetrySparseControlTraversal,
}

/// Structural work retained from successful workflow validation.
///
/// Canonical digest construction is deliberately excluded: it belongs to the
/// separate canonicalization/publication cost lane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplicationWorkflowValidationWork {
    limit_nodes: u64,
    provenance_records: u64,
    indexed_nodes: u64,
    indexed_connections: u64,
    connection_semantics: u64,
    identity_lookups: u64,
    requirement_nodes: u64,
    control_nodes: u64,
    control_edges: u64,
    dominance_candidates: u64,
    dominance_predecessors: u64,
    dominance_table_writes: u64,
    dominance_queries: u64,
    dominance_lifts: u64,
    retry_reachability: u64,
    peak_index_bytes: u64,
}

impl ApplicationWorkflowValidationWork {
    pub const fn complexity_contract(self) -> ApplicationWorkflowValidationComplexityContract {
        ApplicationWorkflowValidationComplexityContract::IndexedSparseGraphWithLogarithmicDominance
    }

    pub const fn indexed_nodes(self) -> u64 {
        self.indexed_nodes
    }

    pub const fn limit_nodes(self) -> u64 {
        self.limit_nodes
    }

    pub const fn provenance_records(self) -> u64 {
        self.provenance_records
    }

    pub const fn indexed_connections(self) -> u64 {
        self.indexed_connections
    }

    pub const fn connection_semantics(self) -> u64 {
        self.connection_semantics
    }

    pub const fn identity_lookups(self) -> u64 {
        self.identity_lookups
    }

    pub const fn requirement_nodes(self) -> u64 {
        self.requirement_nodes
    }

    pub const fn control_nodes(self) -> u64 {
        self.control_nodes
    }

    pub const fn control_edges(self) -> u64 {
        self.control_edges
    }

    pub const fn dominance_predecessors(self) -> u64 {
        self.dominance_predecessors
    }

    pub const fn dominance_candidates(self) -> u64 {
        self.dominance_candidates
    }

    pub const fn dominance_table_writes(self) -> u64 {
        self.dominance_table_writes
    }

    pub const fn dominance_lifts(self) -> u64 {
        self.dominance_lifts
    }

    pub const fn dominance_queries(self) -> u64 {
        self.dominance_queries
    }

    pub const fn retry_reachability(self) -> u64 {
        self.retry_reachability
    }

    pub const fn retry_complexity_contract(
        self,
    ) -> ApplicationWorkflowRetryValidationComplexityContract {
        ApplicationWorkflowRetryValidationComplexityContract::PerRetrySparseControlTraversal
    }

    pub const fn peak_index_bytes(self) -> u64 {
        self.peak_index_bytes
    }

    pub const fn total_visits(self) -> u64 {
        self.limit_nodes
            .saturating_add(self.provenance_records)
            .saturating_add(self.indexed_nodes)
            .saturating_add(self.indexed_connections)
            .saturating_add(self.connection_semantics)
            .saturating_add(self.identity_lookups)
            .saturating_add(self.requirement_nodes)
            .saturating_add(self.control_nodes)
            .saturating_add(self.control_edges)
            .saturating_add(self.dominance_candidates)
            .saturating_add(self.dominance_predecessors)
            .saturating_add(self.dominance_table_writes)
            .saturating_add(self.dominance_queries)
            .saturating_add(self.dominance_lifts)
            .saturating_add(self.retry_reachability)
    }
}

#[derive(Default)]
pub(super) struct ValidationWorkMeter(ApplicationWorkflowValidationWork);

impl ValidationWorkMeter {
    pub(super) fn visit_limit_node(&mut self) {
        self.0.limit_nodes = self.0.limit_nodes.saturating_add(1);
    }

    pub(super) fn visit_provenance_record(&mut self) {
        self.0.provenance_records = self.0.provenance_records.saturating_add(1);
    }

    pub(super) fn index_node(&mut self) {
        self.0.indexed_nodes = self.0.indexed_nodes.saturating_add(1);
    }

    pub(super) fn index_connection(&mut self) {
        self.0.indexed_connections = self.0.indexed_connections.saturating_add(1);
    }

    pub(super) fn visit_connection_semantics(&mut self) {
        self.0.connection_semantics = self.0.connection_semantics.saturating_add(1);
    }

    pub(super) fn lookup_identity(&mut self) {
        self.0.identity_lookups = self.0.identity_lookups.saturating_add(1);
    }

    pub(super) fn visit_requirement_node(&mut self) {
        self.0.requirement_nodes = self.0.requirement_nodes.saturating_add(1);
    }

    pub(super) fn visit_control_node(&mut self) {
        self.0.control_nodes = self.0.control_nodes.saturating_add(1);
    }

    pub(super) fn visit_control_edge(&mut self) {
        self.0.control_edges = self.0.control_edges.saturating_add(1);
    }

    pub(super) fn visit_dominance_predecessor(&mut self) {
        self.0.dominance_predecessors = self.0.dominance_predecessors.saturating_add(1);
    }

    pub(super) fn visit_dominance_candidate(&mut self) {
        self.0.dominance_candidates = self.0.dominance_candidates.saturating_add(1);
    }

    pub(super) fn write_dominance_table(&mut self) {
        self.0.dominance_table_writes = self.0.dominance_table_writes.saturating_add(1);
    }

    pub(super) fn visit_dominance_lift(&mut self) {
        self.0.dominance_lifts = self.0.dominance_lifts.saturating_add(1);
    }

    pub(super) fn visit_dominance_query(&mut self) {
        self.0.dominance_queries = self.0.dominance_queries.saturating_add(1);
    }

    pub(super) fn visit_retry_reachability(&mut self) {
        self.0.retry_reachability = self.0.retry_reachability.saturating_add(1);
    }

    pub(super) fn observe_index_bytes(&mut self, bytes: usize) {
        self.0.peak_index_bytes = self
            .0
            .peak_index_bytes
            .max(u64::try_from(bytes).unwrap_or(u64::MAX));
    }

    pub(super) const fn finish(self) -> ApplicationWorkflowValidationWork {
        self.0
    }
}
