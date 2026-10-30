use worth_signal::facade::{
    DeniedResourceRetry, ResourceBoundaryPerformanceEnvelope, ResourceRetryScheduleReport,
    ScheduledResourceRetry,
};

fn forged_retry() -> ScheduledResourceRetry {
    loop {}
}

fn forged_denial() -> DeniedResourceRetry {
    loop {}
}

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _ = ResourceRetryScheduleReport {
        scheduled_retry: Some(forged_retry()),
        denied_retry: Some(forged_denial()),
        performance: forged_performance(),
    };
}
