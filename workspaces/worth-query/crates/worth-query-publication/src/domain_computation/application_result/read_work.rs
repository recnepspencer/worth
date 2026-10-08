use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryAccessReceipt;

/// Closed descriptive projection of a completed query's actual read work.
///
/// Counts preserve the lower owner's units. They do not describe unique
/// entities, topology construction, allocation, authorization or publication,
/// and cannot authorize a subsequent read.
///
/// ```compile_fail,E0616
/// use worth_query_publication::facade::domain_computation::WorthQueryPublishedApplicationReadWork;
/// fn rewrite(mut work: WorthQueryPublishedApplicationReadWork) {
///     work.total_work_units = 0;
/// }
/// ```
///
/// ```
/// use worth_query_publication::facade::domain_computation::WorthQueryPublishedApplicationReadWork;
/// fn inspect(work: WorthQueryPublishedApplicationReadWork) -> usize {
///     work.total_work_units()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedApplicationReadWork {
    predicate_work_units: usize,
    adjacency_work_units: usize,
    ordering_work_units: usize,
    continuation_seek_work_units: usize,
    projection_work_units: usize,
    total_work_units: usize,
    examined_candidate_count: usize,
    projected_record_count: usize,
    projected_field_count: usize,
    adjacency_list_read_count: usize,
    edge_scan_count: usize,
    ordering_comparison_count: usize,
    ordered_index_entry_count: usize,
    target_identity_index_entry_count: usize,
    per_result_neighbor_lookup_count: usize,
    fallback_count: usize,
    truncation_count: usize,
}

impl WorthQueryPublishedApplicationReadWork {
    pub(super) fn capture(terminal: &WorthQueryApplicationQueryAccessReceipt) -> Self {
        let work = terminal.work();
        Self {
            predicate_work_units: work.predicate_work_units(),
            adjacency_work_units: work.adjacency_work_units(),
            ordering_work_units: work.ordering_work_units(),
            continuation_seek_work_units: work.continuation_seek_work_units(),
            projection_work_units: work.projection_work_units(),
            total_work_units: work.total_work_units(),
            examined_candidate_count: terminal.examined_candidate_count(),
            projected_record_count: terminal.projected_record_count(),
            projected_field_count: terminal.projected_field_count(),
            adjacency_list_read_count: terminal.adjacency_list_read_count(),
            edge_scan_count: terminal.edge_scan_count(),
            ordering_comparison_count: terminal.ordering_comparison_count(),
            ordered_index_entry_count: terminal.ordered_index_entry_count(),
            target_identity_index_entry_count: terminal.target_identity_index_entry_count(),
            per_result_neighbor_lookup_count: terminal.per_result_neighbor_lookup_count(),
            fallback_count: terminal.fallback_count(),
            truncation_count: terminal.truncation_count(),
        }
    }

    pub const fn predicate_work_units(self) -> usize {
        self.predicate_work_units
    }
    pub const fn adjacency_work_units(self) -> usize {
        self.adjacency_work_units
    }
    pub const fn ordering_work_units(self) -> usize {
        self.ordering_work_units
    }
    pub const fn continuation_seek_work_units(self) -> usize {
        self.continuation_seek_work_units
    }
    pub const fn projection_work_units(self) -> usize {
        self.projection_work_units
    }
    pub const fn total_work_units(self) -> usize {
        self.total_work_units
    }
    pub const fn examined_candidate_count(self) -> usize {
        self.examined_candidate_count
    }
    pub const fn projected_record_count(self) -> usize {
        self.projected_record_count
    }
    pub const fn projected_field_count(self) -> usize {
        self.projected_field_count
    }
    pub const fn adjacency_list_read_count(self) -> usize {
        self.adjacency_list_read_count
    }
    pub const fn edge_scan_count(self) -> usize {
        self.edge_scan_count
    }
    pub const fn ordering_comparison_count(self) -> usize {
        self.ordering_comparison_count
    }
    pub const fn ordered_index_entry_count(self) -> usize {
        self.ordered_index_entry_count
    }
    pub const fn target_identity_index_entry_count(self) -> usize {
        self.target_identity_index_entry_count
    }
    pub const fn per_result_neighbor_lookup_count(self) -> usize {
        self.per_result_neighbor_lookup_count
    }
    pub const fn fallback_count(self) -> usize {
        self.fallback_count
    }
    pub const fn truncation_count(self) -> usize {
        self.truncation_count
    }
}
