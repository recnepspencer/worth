use worth_store::physical_runtime::ServingPhysicalRuntime;

fn evict(serving: &ServingPhysicalRuntime) {
    serving.evict_resident_frame();
}

fn main() {}
