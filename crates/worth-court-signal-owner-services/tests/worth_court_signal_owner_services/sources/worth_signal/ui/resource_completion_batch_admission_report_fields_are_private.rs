use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceCompletionBatchAdmissionReport,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceCompletionBatchAdmissionReport {
        admitted_completions: Vec::new(),
        denied_completions: Vec::new(),
        input_width: 0,
        deduplicated_width: 0,
        duplicate_width: 0,
        performance: forged_performance(),
    };
}
