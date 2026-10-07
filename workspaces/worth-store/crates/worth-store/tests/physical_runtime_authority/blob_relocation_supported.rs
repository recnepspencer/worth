use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedRecordPlacementPolicy, BlobReadLimits, PhysicalExtentCopyPhase,
    PhysicalMutationRequest, PublishedBlobGeneration, ServingPhysicalRuntime,
};

fn move_selected_chunk(
    runtime: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    scope: &AdmittedBlobScope,
    limits: BlobReadLimits,
    target: AdmittedRecordPlacementPolicy,
    request: PhysicalMutationRequest,
) {
    let blobs = runtime.blobs().unwrap();
    let hold = blobs
        .hold_chunk_for_relocation(published, scope, 0, limits)
        .unwrap();
    let mut movement = blobs.begin_chunk_relocation(hold, target, request).unwrap();
    for _ in 0..512 {
        if movement.progress().phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        movement.advance().unwrap();
    }
    assert_eq!(
        movement.progress().phase,
        PhysicalExtentCopyPhase::ReadyForAdoption
    );
    let receipt = movement.publish().unwrap();
    assert!(receipt.logical_chunk_bytes() > 0);
}

fn main() {
    std::hint::black_box(move_selected_chunk);
}
