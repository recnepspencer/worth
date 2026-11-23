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
            counters.count_preparation_packet_shape(packets, items, max_width, scope_units);
        });
    }

    pub(crate) fn count_preparation_parallel_legal(&self) {
        self.counters.count(
            crate::performance::data::RuntimeComplexityCounters::count_preparation_parallel_legal,
        );
    }

    pub(crate) fn count_preparation_parallel_profitable(&self) {
        self.counters
            .count(crate::performance::data::RuntimeComplexityCounters::count_preparation_parallel_profitable);
    }

    pub(crate) fn count_preparation_serial_strategy(&self) {
        self.counters.count(
            crate::performance::data::RuntimeComplexityCounters::count_preparation_serial_strategy,
        );
    }

    pub(crate) fn count_preparation_staged_parallel_strategy(&self) {
        self.counters
            .count(crate::performance::data::RuntimeComplexityCounters::count_preparation_staged_parallel_strategy);
    }

    pub(crate) fn count_preparation_reducer_conflicts(&self, conflicts: usize) {
        self.counters
            .count(|counters| counters.count_preparation_reducer_conflicts(conflicts));
    }
}
