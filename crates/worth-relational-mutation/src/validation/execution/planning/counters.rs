use crate::authority::commit::preparation::diagnostics::counters::ValidationPreparationCounters;
use crate::authority::commit::preparation::facade::PreparedInvariantExecution;

pub(crate) fn planned_packet_counters(
    planned: &PreparedInvariantExecution<'_>,
) -> ValidationPreparationCounters {
    ValidationPreparationCounters {
        packet_count: planned.packets.len(),
        worker_result_count: 0,
        reducer_input_count: 0,
        reducer_conflict_count: 0,
        failure_count: 0,
    }
}
