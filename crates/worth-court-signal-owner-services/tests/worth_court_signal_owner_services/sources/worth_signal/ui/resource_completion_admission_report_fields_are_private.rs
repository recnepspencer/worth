use worth_signal::facade::{
    AdmittedResourceCompletion, DeniedResourceCompletion, ResourceBoundaryPerformanceEnvelope,
    ResourceCompletionAdmissionReport,
};

fn forged_admitted() -> AdmittedResourceCompletion {
    loop {}
}

fn forged_denied() -> DeniedResourceCompletion {
    loop {}
}

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _ = ResourceCompletionAdmissionReport {
        admitted_completion: Some(forged_admitted()),
        denied_completion: Some(forged_denied()),
        performance: forged_performance(),
    };
}
