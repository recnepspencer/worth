use worth_signal::facade::core::ResourceObservationBatchReport;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _report = ResourceObservationBatchReport {
        events: fake(),
        performance: fake(),
    };
}
