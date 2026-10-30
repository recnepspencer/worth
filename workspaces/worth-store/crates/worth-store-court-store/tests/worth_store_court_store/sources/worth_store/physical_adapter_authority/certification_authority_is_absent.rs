use worth_store::physical_runtime::ServingPhysicalRuntime;

fn reach_certification(serving: &ServingPhysicalRuntime) {
    let _ = serving.certification_physical_residency();
}

fn main() {}
