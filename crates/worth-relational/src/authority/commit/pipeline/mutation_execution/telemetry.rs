use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub(super) fn record_preparation_telemetry(
    instrumentation: &RuntimeInstrumentation,
    telemetry: crate::authority::mutation::MutationPreparationTelemetry,
) {
    instrumentation.count(|counters| {
        counters.count_preparation_packet_shape(
            telemetry.packet_count,
            telemetry.packet_item_count,
            telemetry.packet_peak_width_total,
            telemetry.scope_unit_count,
        );
    });
    for _ in 0..telemetry.parallel_legal_count {
        instrumentation.count(RuntimeComplexityCounters::count_preparation_parallel_legal);
    }
    for _ in 0..telemetry.parallel_profitable_count {
        instrumentation.count(RuntimeComplexityCounters::count_preparation_parallel_profitable);
    }
    for _ in 0..telemetry.serial_strategy_count {
        instrumentation.count(RuntimeComplexityCounters::count_preparation_serial_strategy);
    }
    for _ in 0..telemetry.staged_parallel_strategy_count {
        instrumentation
            .count(RuntimeComplexityCounters::count_preparation_staged_parallel_strategy);
    }
}
