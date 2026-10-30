use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceRuntimeSummary, ResourceRuntimeSummaryReadReport,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceRuntimeSummaryReadReport {
        summary: ResourceRuntimeSummary::default(),
        performance: forged_performance(),
    };
}
