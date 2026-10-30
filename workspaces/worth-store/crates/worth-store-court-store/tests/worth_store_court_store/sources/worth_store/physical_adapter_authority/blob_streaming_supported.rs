use worth_store::physical_runtime::{
    PhysicalOperationAllocationScope, PhysicalResidencyObservation, RecordReadSession,
    RecordStreamFailure,
};

#[expect(
    dead_code,
    reason = "Positive compile-only consumer/type projection witness; the complete signature and body are checked without fabricating owner-issued validated inputs"
)]
fn admitted_blob_scope(observation: PhysicalResidencyObservation) -> u64 {
    observation
        .admitted_policy()
        .scope_bytes(PhysicalOperationAllocationScope::Blob)
}

#[expect(
    dead_code,
    reason = "Positive compile-only consumer/type projection witness; the complete signature and body are checked without fabricating owner-issued validated inputs"
)]
fn stream_blob_chunks(
    mut session: RecordReadSession,
    mut consume: impl FnMut(&[u8]),
) -> Result<(), RecordStreamFailure> {
    while let Some(chunk) = session.next_chunk()? {
        consume(chunk.bytes());
    }
    Ok(())
}

pub(crate) fn main() {}
