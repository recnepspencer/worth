use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceDiagnosticsSummary,
    ResourceDiagnosticsExpansionBudget, ResourceReplayReconstructionReport, ResourceRuntimeSummary,
};

fn forged_replay() -> ResourceReplayReconstructionReport {
    loop {}
}

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceDiagnosticsSummary {
        schema_version: String::new(),
        runtime_summary: ResourceRuntimeSummary::default(),
        latest_branch_restore_report: None,
        replay_reconstruction: forged_replay(),
        expansion_budget: ResourceDiagnosticsExpansionBudget::retained_summary_only(),
        performance: forged_performance(),
        provenance_digest: String::new(),
    };
}
