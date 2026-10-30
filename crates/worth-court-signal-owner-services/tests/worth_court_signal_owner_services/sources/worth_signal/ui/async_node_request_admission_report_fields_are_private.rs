use worth_signal::facade::AsyncNodeRequestAdmissionReport;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _report = AsyncNodeRequestAdmissionReport {
        classification: fake(),
        resource_admission: fake(),
    };
}
