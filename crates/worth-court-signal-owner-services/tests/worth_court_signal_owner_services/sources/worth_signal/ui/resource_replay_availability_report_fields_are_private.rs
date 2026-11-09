use worth_signal::facade::ResourceReplayAvailabilityReport;

fn main() {
    let _ = ResourceReplayAvailabilityReport {
        class: supplied(),
        retained_history_unavailable_count: 0,
        denied_completion_unavailable_count: 0,
        retry_lineage_unavailable_count: 0,
        summary_read: supplied(),
        restore_compatibility: None,
        restore_compatibility_denial: None,
        diagnostics_summary: None,
        diagnostics_denial: None,
        availability_digest: String::new(),
        performance: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
