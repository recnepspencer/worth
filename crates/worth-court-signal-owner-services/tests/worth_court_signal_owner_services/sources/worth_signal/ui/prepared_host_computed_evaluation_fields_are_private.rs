use worth_signal::facade::PreparedHostComputedEvaluation;

fn main() {
    let _ = PreparedHostComputedEvaluation {
        request: supplied(),
        evaluation: supplied(),
        admitted_reads: supplied(),
        dependency_patch: supplied(),
        diagnostics_summary: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
