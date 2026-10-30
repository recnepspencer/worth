use worth_signal::facade::core::AsyncNodeHistoricalParityReport;

fn parity_report() -> AsyncNodeHistoricalParityReport {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let report = parity_report();
    let _ = report.node;
}
