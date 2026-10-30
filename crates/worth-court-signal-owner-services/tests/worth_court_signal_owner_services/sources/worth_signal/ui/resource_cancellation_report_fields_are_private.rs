use worth_signal::facade::{
    CancelledResourceRequest, ResourceBoundaryPerformanceEnvelope, ResourceCancellationReport,
    ResourceDependentCancellationPropagation, ResourceLifecycleSummary,
    ResourceLifecycleTransition,
};

fn forged_cancelled() -> CancelledResourceRequest {
    loop {}
}

fn forged_lifecycle() -> ResourceLifecycleSummary {
    loop {}
}

fn forged_propagation() -> ResourceDependentCancellationPropagation {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _ = ResourceCancellationReport {
        cancelled_request: Some(forged_cancelled()),
        dependent_propagation: Some(forged_propagation()),
        denied_cancellation: None,
        lifecycle: Some(forged_lifecycle()),
        transition: Some(forged_transition()),
        performance: forged_performance(),
    };
}
