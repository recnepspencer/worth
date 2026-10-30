use worth_signal::facade::core::AsyncNodeCapabilityEquivalenceReport;

fn equivalence_report() -> AsyncNodeCapabilityEquivalenceReport {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let report = equivalence_report();
    let _ = report.node;
}
