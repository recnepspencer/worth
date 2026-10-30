use worth_signal::facade::PreparedHostComputedEvaluation;

fn main() {
    let _ = PreparedHostComputedEvaluation {
        request: loop {},
        evaluation: loop {},
        admitted_reads: loop {},
        dependency_patch: loop {},
        diagnostics_summary: loop {},
    };
}
