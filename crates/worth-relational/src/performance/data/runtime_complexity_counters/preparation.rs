use super::RuntimeComplexityCounters;

impl RuntimeComplexityCounters {
    pub(crate) fn count_preparation_packet_shape(
        &mut self,
        packets: usize,
        items: usize,
        max_width: usize,
        scope_units: usize,
    ) {
        self.preparation_packet_count += packets;
        self.preparation_packet_item_count += items;
        self.preparation_packet_peak_width_total += max_width;
        self.preparation_scope_unit_count += scope_units;
    }

    pub(crate) fn count_preparation_parallel_legal(&mut self) {
        self.preparation_parallel_legal_count += 1;
    }

    pub(crate) fn count_preparation_parallel_profitable(&mut self) {
        self.preparation_parallel_profitable_count += 1;
    }

    pub(crate) fn count_preparation_serial_strategy(&mut self) {
        self.preparation_serial_strategy_count += 1;
    }

    pub(crate) fn count_preparation_staged_parallel_strategy(&mut self) {
        self.preparation_staged_parallel_strategy_count += 1;
    }

    pub(crate) fn count_preparation_reducer_conflicts(&mut self, conflicts: usize) {
        self.preparation_reducer_conflict_count += conflicts;
    }
}
