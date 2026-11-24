#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ValidationPreparationCounters {
    pub(crate) packet_count: usize,
    pub(crate) worker_result_count: usize,
    pub(crate) reducer_input_count: usize,
    pub(crate) reducer_conflict_count: usize,
    pub(crate) failure_count: usize,
}
