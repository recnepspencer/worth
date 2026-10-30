use worth_store::physical_runtime::{
    PhysicalRecordChunkBasis, PhysicalRecordChunkView, RecordReadSession, RecordStreamFailure,
};

fn consume_inline(session: RecordReadSession) -> Result<(), RecordStreamFailure> {
    consume_chunks(session)
}

fn consume_extent(session: RecordReadSession) -> Result<(), RecordStreamFailure> {
    consume_chunks(session)
}

fn consume_chunks(mut session: RecordReadSession) -> Result<(), RecordStreamFailure> {
    while let Some(chunk) = session.next_chunk()? {
        observe(chunk);
    }
    Ok(())
}

fn observe(chunk: PhysicalRecordChunkView<'_>) {
    let _: &[u8] = chunk.bytes();
    let _: std::ops::Range<u64> = chunk.logical_range();
    let basis: PhysicalRecordChunkBasis = chunk.basis();
    let _ = basis.store_identity();
    let _ = basis.store_generation();
    let _ = basis.record();
    let _ = basis.frame_coordinate();
}

fn main() {
    let _ = consume_inline;
    let _ = consume_extent;
}
