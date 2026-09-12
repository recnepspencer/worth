use worth_store::physical_runtime::ServingPhysicalRuntime;

fn borrow_product_capabilities(serving: &ServingPhysicalRuntime) {
    let records = serving.records().expect("read protection admission");
    let submissions = serving.record_submission();
    let residency = serving.residency_observation();

    let _ = (
        records.store_identity(),
        submissions,
        residency.store_identity(),
        residency.store_generation(),
        residency.counters(),
    );
}

fn main() {}
