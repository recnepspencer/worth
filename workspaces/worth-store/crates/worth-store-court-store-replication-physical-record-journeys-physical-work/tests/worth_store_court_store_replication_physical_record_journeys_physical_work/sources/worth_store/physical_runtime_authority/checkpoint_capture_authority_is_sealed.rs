use worth_store::physical_runtime::{CompletedPhysicalCheckpoint, PhysicalCheckpointCaptureBasis};
use worth_store_physical_format::CheckpointStreamFooter;

fn forge_completion(
    basis: PhysicalCheckpointCaptureBasis,
    footer: CheckpointStreamFooter,
) -> CompletedPhysicalCheckpoint {
    CompletedPhysicalCheckpoint {
        basis,
        footer,
        encoded_bytes: 0,
        dirty_records: 0,
    }
}

fn main() {}
