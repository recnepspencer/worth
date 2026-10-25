use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_preparation_packet_shape(
        &self,
        packets: usize,
        items: usize,
        max_width: usize,
        scope_units: usize,
    ) {
        self.counters.count(|counters| {
            counters.preparation_packet_count += packets;
            counters.preparation_packet_item_count += items;
            counters.preparation_packet_peak_width_total += max_width;
            counters.preparation_scope_unit_count += scope_units;
        });
    }

    pub(crate) fn count_preparation_parallel_legal(&self) {
        self.counters
            .count(|counters| counters.preparation_parallel_legal_count += 1);
    }

    pub(crate) fn count_preparation_parallel_profitable(&self) {
        self.counters
            .count(|counters| counters.preparation_parallel_profitable_count += 1);
    }

    pub(crate) fn count_preparation_serial_strategy(&self) {
        self.counters
            .count(|counters| counters.preparation_serial_strategy_count += 1);
    }

    pub(crate) fn count_preparation_staged_parallel_strategy(&self) {
        self.counters
            .count(|counters| counters.preparation_staged_parallel_strategy_count += 1);
    }

    pub(crate) fn count_preparation_reducer_conflicts(&self, conflicts: usize) {
        self.counters
            .count(|counters| counters.preparation_reducer_conflict_count += conflicts);
    }
}
