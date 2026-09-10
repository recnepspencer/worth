#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryConsumerInvalidationCounters {
    pub lease_impact_readmission_attempts: usize,
    pub semantic_delivery_checks: usize,
    pub consumer_support_checks: usize,
    pub disposition_classifications: usize,
    pub native_access_layout_lookups: usize,
    pub native_key_index_lookups: usize,
    pub native_path_index_probes: usize,
    pub targeted_native_key_visits: usize,
    pub native_key_overlap_deduplications: usize,
    pub targeted_lease_deliveries: usize,
    pub live_source_authority_checks: usize,
    pub delta_authority_readmission_checks: usize,
    pub epoch_readmission_checks: usize,
    pub sharing_readmission_checks: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryConsumerInvalidationEpochCounters {
    pub capability_index_lookups: usize,
    pub live_collection_index_probes: usize,
    pub live_relevance_index_probes: usize,
    pub installed_collection_index_probes: usize,
    pub installed_relevance_index_probes: usize,
    pub live_target_candidates_visited: usize,
    pub installed_target_candidates_selected: usize,
    pub installed_candidates_skipped: usize,
    pub target_overlap_deduplications: usize,
    pub installed_route_index_probes: usize,
    pub fanout_targets: usize,
    pub delivery_batches_visited: usize,
    pub mutation_deltas_visited: usize,
    pub touched_aspects_visited: usize,
}
