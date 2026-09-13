use worth_store_physical_format::physical_work_obligation::decode_physical_work_obligation_v6;
use worth_store_physical_format::wal_frame::decode_bounded_wal_frame_v1;
use worth_store_physical_format::{
    decode_checkpoint_binding_record, decode_extent_chunk, inspect_inline_page,
    CheckpointBindingCompactionHeader,
    CheckpointDirtyFrameBasis, CheckpointStreamFooter, DurablePhysicalRootManifest,
    DurableExtentManifest, DurableRootSelector, ExtentChunkCoordinate, PhysicalCheckpointSource,
};
use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader,
    IntegrityValidatedCurrentRootSelector, IntegrityValidatedPreviousRootSelector,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPageFrame, IntegrityValidatedPhysicalWorkObligation,
    IntegrityValidatedRootManifest, IntegrityValidatedWalFrame,
};

fn decode_checkpoint_header(validated: IntegrityValidatedCheckpointStreamHeader<'_>) {
    let _ = PhysicalCheckpointSource::decode_stream_header_record(validated);
}

fn decode_checkpoint_dirty(validated: IntegrityValidatedCheckpointDirtyBasis<'_>) {
    let _ = CheckpointDirtyFrameBasis::decode_record(validated);
}

fn decode_checkpoint_compaction(validated: IntegrityValidatedCheckpointBindingCompaction<'_>) {
    let _ = CheckpointBindingCompactionHeader::decode_record(validated);
}

fn decode_checkpoint_binding(validated: IntegrityValidatedCheckpointBinding<'_>) {
    let _ = decode_checkpoint_binding_record(validated);
}

fn decode_checkpoint_footer(validated: IntegrityValidatedCheckpointFooter<'_>) {
    let _ = CheckpointStreamFooter::decode_record(validated);
}

fn decode_selector(validated: IntegrityValidatedCurrentRootSelector<'_>) {
    let _ = DurableRootSelector::decode(validated);
}

fn decode_previous_selector(validated: IntegrityValidatedPreviousRootSelector<'_>) {
    let _ = DurableRootSelector::decode(validated);
}

fn decode_manifest(validated: IntegrityValidatedRootManifest<'_>) {
    let _ = DurablePhysicalRootManifest::decode(validated, 2);
}

fn decode_physical_work(validated: IntegrityValidatedPhysicalWorkObligation<'_>) {
    let _ = decode_physical_work_obligation_v6(validated);
}

fn decode_page(validated: IntegrityValidatedPageFrame<'_>) {
    let _ = inspect_inline_page(validated.record_format(), validated);
}

fn decode_wal(validated: IntegrityValidatedWalFrame<'_>) {
    let _ = decode_bounded_wal_frame_v1(validated);
}

fn decode_extent_manifest(validated: IntegrityValidatedExtentManifest<'_>) {
    let _ = DurableExtentManifest::decode(validated);
}

fn decode_extent(
    validated: IntegrityValidatedExtentChunkFrame<'_>,
    expected: ExtentChunkCoordinate,
) {
    let _ = decode_extent_chunk(validated, expected);
}

fn extract_extent_payload(validated: IntegrityValidatedExtentChunkFrame<'_>) {
    let _ = validated.chunk_bytes();
}

fn main() {}
