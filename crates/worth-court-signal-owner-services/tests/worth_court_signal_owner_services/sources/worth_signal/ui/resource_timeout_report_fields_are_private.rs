use worth_signal::facade::{
    DeniedResourceTimeout, ResourceBoundaryPerformanceEnvelope, ResourceLifecycleSummary,
    ResourceLifecycleTransition, ResourceTimeoutReport, TimedOutResourceRequest,
};

fn forged_timeout() -> TimedOutResourceRequest {
    loop {}
}

fn forged_denial() -> DeniedResourceTimeout {
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
    let _ = ResourceTimeoutReport {
        timed_out_request: Some(forged_timeout()),
        denied_timeout: Some(forged_denial()),
        lifecycle: Some(forged_lifecycle()),
        transition: Some(forged_transition()),
        performance: forged_performance(),
    };
}
