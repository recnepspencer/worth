use worth_signal::facade::ResourceOverlappingGenerationAdmission;

fn main() {
    let _ = std::mem::size_of::<ResourceOverlappingGenerationAdmission>();
    let _ = ResourceOverlappingGenerationAdmission {
        previous: supplied(),
        replacing: supplied(),
        policy_decision_digest: supplied(),
        old_host_work_cancellation_advisory: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
