use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_query_packet_shape(
        &self,
        packets: usize,
        items: usize,
        max_width: usize,
        scope_units: usize,
    ) {
        self.counters.count(|counters| {
            counters.query_packet_count += packets;
            counters.query_packet_item_count += items;
            counters.query_packet_peak_width_total += max_width;
            counters.query_scope_unit_count += scope_units;
        });
    }

    pub(crate) fn count_query_parallel_legal(&self) {
        self.counters
            .count(|counters| counters.query_parallel_legal_count += 1);
    }

    pub(crate) fn count_query_parallel_profitable(&self) {
        self.counters
            .count(|counters| counters.query_parallel_profitable_count += 1);
    }

    pub(crate) fn count_query_serial_strategy(&self) {
        self.counters
            .count(|counters| counters.query_serial_strategy_count += 1);
    }

    pub(crate) fn count_query_staged_parallel_strategy(&self) {
        self.counters
            .count(|counters| counters.query_staged_parallel_strategy_count += 1);
    }

    pub(crate) fn count_query_fragment_scratch_reuse_by(&self, count: usize) {
        self.counters.count(|counters| {
            counters.query_fragment_scratch_reuse_count += count;
        });
    }

    pub(crate) fn count_query_index_attempt(&self) {
        self.counters
            .count(|counters| counters.query_index_attempt_count += 1);
    }

    pub(crate) fn count_query_index_path(&self) {
        self.counters
            .count(|counters| counters.query_index_path_count += 1);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_rejection(&self) {
        self.counters
            .count(|counters| counters.query_index_rejection_count += 1);
    }

    pub(crate) fn count_query_index_parity_verification(&self) {
        self.counters
            .count(|counters| counters.query_index_parity_verification_count += 1);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_scratch_reuse(&self) {
        self.counters
            .count(|counters| counters.query_index_scratch_reuse_count += 1);
    }

    pub(crate) fn count_query_emissions(&self, entities: usize, relations: usize) {
        self.counters.count(|counters| {
            counters.query_authoritative_entity_records_emitted += entities;
            counters.query_authoritative_relation_records_emitted += relations;
        });
    }
}
