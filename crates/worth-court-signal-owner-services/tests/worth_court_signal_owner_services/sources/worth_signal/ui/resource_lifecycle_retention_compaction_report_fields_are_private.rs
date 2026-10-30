use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceLifecycleRetentionCompactionReport,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceLifecycleRetentionCompactionReport {
        selected_terminal_count: 1,
        reclaimed_in_flight_count: 1,
        retained_history_write_count: 1,
        retained_history_pruned_count: 0,
        retained_history_unavailable_count: 0,
        retained_history_width: 1,
        hot_in_flight_width: 0,
        compacted_terminal_summary_count: 0,
        compacted_superseded_count: 0,
        compacted_cancelled_count: 0,
        compacted_timed_out_count: 0,
        performance: forged_performance(),
    };
}
