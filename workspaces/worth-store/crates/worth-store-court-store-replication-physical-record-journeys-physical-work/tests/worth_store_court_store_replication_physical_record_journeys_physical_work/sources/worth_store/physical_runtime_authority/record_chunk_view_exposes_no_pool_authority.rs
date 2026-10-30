use worth_store::physical_runtime::PhysicalRecordChunkView;

fn reach_pool(view: PhysicalRecordChunkView<'_>) {
    let _: &worth_store_buffer_pool::PhysicalFrameLease = &*view;
}

fn main() {}
