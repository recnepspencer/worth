use worth_store::physical_runtime::{LifecycleGeneration, PhysicalRecordId};
use worth_store::physical_runtime::{PhysicalRecordChunkBasis, PhysicalRecordChunkView};
use worth_store_physical_format::{store_namespace::StableStoreIdentity, RecordFrameCoordinate};

fn fabricate_basis(
    store: StableStoreIdentity,
    generation: LifecycleGeneration,
    record: PhysicalRecordId,
    frame: RecordFrameCoordinate,
) -> PhysicalRecordChunkBasis {
    PhysicalRecordChunkBasis::new(store, generation, record, frame)
}

fn fabricate_view<'a>(
    bytes: &'a [u8],
    basis: PhysicalRecordChunkBasis,
) -> PhysicalRecordChunkView<'a> {
    PhysicalRecordChunkView::new(bytes, basis, 0..bytes.len() as u64)
}

fn main() {}
