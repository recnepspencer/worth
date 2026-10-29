use worth_signal::facade::core::AsyncKeyedNodeCapabilityEquivalenceReport;

fn report() -> AsyncKeyedNodeCapabilityEquivalenceReport {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let report = report();
    let _ = report.family;
}
