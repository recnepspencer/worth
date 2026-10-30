use worth_signal::facade::{
    DeniedResourceRejection, RejectedResourceRequest, ResourceBoundaryPerformanceEnvelope,
    ResourceLifecycleSummary, ResourceLifecycleTransition, ResourceRejectionReport,
};

fn forged_rejected() -> RejectedResourceRequest {
    loop {}
}

fn forged_denial() -> DeniedResourceRejection {
    loop {}
}

fn forged_lifecycle() -> ResourceLifecycleSummary {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _ = ResourceRejectionReport {
        rejected_request: Some(forged_rejected()),
        denied_rejection: Some(forged_denial()),
        lifecycle: Some(forged_lifecycle()),
        transition: Some(forged_transition()),
        performance: forged_performance(),
    };
}
