use worth_store::physical_runtime::{
    PhysicalOperationAllocationScope, PhysicalResidencyObservation, RecordReadSession,
    RecordStreamFailure,
};

fn admitted_blob_scope(observation: PhysicalResidencyObservation) -> u64 {
    observation
        .admitted_policy()
        .scope_bytes(PhysicalOperationAllocationScope::Blob)
}

fn stream_blob_chunks(
    mut session: RecordReadSession,
    mut consume: impl FnMut(&[u8]),
) -> Result<(), RecordStreamFailure> {
    while let Some(chunk) = session.next_chunk()? {
        consume(chunk.bytes());
    }
    Ok(())
}

fn main() {}
