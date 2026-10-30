use worth_store::physical_runtime::PhysicalRecordChunkBasis;

fn unavailable<T>() -> T {
    loop {
        std::hint::spin_loop();
    }
}

fn forge_generation() -> PhysicalRecordChunkBasis {
    PhysicalRecordChunkBasis {
        store: unavailable(),
        generation: unavailable(),
        record: unavailable(),
        frame: unavailable(),
    }
}

fn main() {}
