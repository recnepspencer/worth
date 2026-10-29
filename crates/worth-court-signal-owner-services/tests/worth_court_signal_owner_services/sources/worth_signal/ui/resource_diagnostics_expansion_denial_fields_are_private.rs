use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceDiagnosticsExpansionBudget,
    ResourceDiagnosticsExpansionDenial, ResourceDiagnosticsExpansionDenialClass,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceDiagnosticsExpansionDenial {
        class: ResourceDiagnosticsExpansionDenialClass::ColdReconstructionDisabled,
        budget: ResourceDiagnosticsExpansionBudget::retained_summary_only(),
        replay_reconstruction_width: 1,
        performance: forged_performance(),
    };
}
