use worth_signal::facade::core::AsyncKeyedNodeHistoricalParityReport;

fn report() -> AsyncKeyedNodeHistoricalParityReport {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let report = report();
    let _ = report.family;
}
