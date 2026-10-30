use worth_store_physical_backend::{
    ArtifactRangeWriteDurability, CompletedArtifactRangeWrite,
};

fn main() {
    let _ = CompletedArtifactRangeWrite {
        owner: unsafe { core::mem::zeroed() },
        store: unsafe { core::mem::zeroed() },
        coordinate: unsafe { core::mem::zeroed() },
        payload_digest: [0; 32],
        completed_bytes: 0,
        operation: unsafe { core::mem::zeroed() },
        durability: ArtifactRangeWriteDurability::BufferedWriteCompleted,
    };
}
