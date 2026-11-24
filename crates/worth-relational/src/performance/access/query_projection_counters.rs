use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_query_serial_strategy(&self) {
        self.counters
            .count(|counters| counters.query_serial_strategy_count += 1);
    }

    pub(crate) fn count_query_staged_parallel_strategy(&self) {
        self.counters
            .count(|counters| counters.query_staged_parallel_strategy_count += 1);
    }

    pub(crate) fn count_query_index_attempt(&self) {
        self.counters
            .count(crate::performance::data::RuntimeComplexityCounters::count_query_index_attempt);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_path(&self) {
        self.counters
            .count(crate::performance::data::RuntimeComplexityCounters::count_query_index_path);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_rejection(&self) {
        self.counters
            .count(|counters| counters.query_index_rejection_count += 1);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_parity_verification(&self) {
        self.counters.count(crate::performance::data::RuntimeComplexityCounters::count_query_index_parity_verification);
    }

    #[cfg(test)]
    pub(crate) fn count_query_index_scratch_reuse(&self) {
        self.counters
            .count(|counters| counters.query_index_scratch_reuse_count += 1);
    }
}
