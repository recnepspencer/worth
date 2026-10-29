use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceBranchRestoreReport,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceBranchRestoreReport {
        restored_in_flight_width: 1,
        retained_summary_width: 1,
        broad_rebuild_denial_count: 1,
        performance: forged_performance(),
    };
}
