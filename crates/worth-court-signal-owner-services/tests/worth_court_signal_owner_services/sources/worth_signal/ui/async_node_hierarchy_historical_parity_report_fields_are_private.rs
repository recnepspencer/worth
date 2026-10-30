use worth_signal::facade::core::AsyncNodeHierarchyHistoricalParityReport;

fn report() -> AsyncNodeHierarchyHistoricalParityReport {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let report = report();
    let _ = report.root_node;
}
