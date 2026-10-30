use worth_store::physical_runtime::{
    stability::StablePhysicalReadReceipt, AdmittedRecordPlacementPolicy, BlobMovementReadHold,
    PhysicalMutationRequest, ServingPhysicalRuntime,
};

fn substitute_hold(receipt: StablePhysicalReadReceipt) -> BlobMovementReadHold<'static> {
    receipt
}

fn substitute_execution(
    runtime: &ServingPhysicalRuntime,
    receipt: StablePhysicalReadReceipt,
    target: AdmittedRecordPlacementPolicy,
    request: PhysicalMutationRequest,
) {
    let _ = runtime
        .blobs()
        .unwrap()
        .begin_chunk_relocation(receipt, target, request);
}

fn main() {}
