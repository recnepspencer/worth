use worth_signal::facade::ResourceReplayAvailabilityReport;

fn main() {
    let _ = ResourceReplayAvailabilityReport {
        class: todo!(),
        retained_history_unavailable_count: 0,
        denied_completion_unavailable_count: 0,
        retry_lineage_unavailable_count: 0,
        summary_read: todo!(),
        restore_compatibility: None,
        restore_compatibility_denial: None,
        diagnostics_summary: None,
        diagnostics_denial: None,
        availability_digest: String::new(),
        performance: todo!(),
    };
}
