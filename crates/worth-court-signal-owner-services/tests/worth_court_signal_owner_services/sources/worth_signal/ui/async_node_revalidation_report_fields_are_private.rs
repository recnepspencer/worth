use worth_signal::facade::AsyncNodeRevalidationReport;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _report = AsyncNodeRevalidationReport {
        classification: fake(),
        resource_revalidation: fake(),
    };
}
