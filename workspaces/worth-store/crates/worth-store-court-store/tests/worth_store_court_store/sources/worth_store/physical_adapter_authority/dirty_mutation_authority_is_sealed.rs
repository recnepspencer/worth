use worth_store::physical_runtime::ServingPhysicalRuntime;

fn mutate_dirty_state(serving: &ServingPhysicalRuntime) {
    serving.mark_resident_frame_dirty();
}

fn main() {}
