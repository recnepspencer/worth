use worth_store_physical_format::wal_frame::decode_bounded_wal_frame_v1;
use worth_store_physical_format::{
    decode_checkpoint_binding_record, decode_extent_chunk, inspect_inline_page,
    ExtentChunkCoordinate, PhysicalRecordFormatDeclaration,
};
use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBindingPayloadProjection,
    IntegrityValidatedExtentChunkProjection, IntegrityValidatedInlineRecordProjection,
    IntegrityValidatedWalPayloadProjection,
};

fn decode_page(projected: IntegrityValidatedInlineRecordProjection<'_, '_>) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let _ = inspect_inline_page(format, projected);
}

fn decode_extent(
    projected: IntegrityValidatedExtentChunkProjection<'_, '_>,
    expected: ExtentChunkCoordinate,
) {
    let _ = decode_extent_chunk(projected, expected);
}

fn decode_wal(projected: IntegrityValidatedWalPayloadProjection<'_, '_>) {
    let _ = decode_bounded_wal_frame_v1(projected);
}

fn decode_checkpoint(
    projected: IntegrityValidatedCheckpointBindingPayloadProjection<'_, '_>,
) {
    let _ = decode_checkpoint_binding_record(projected);
}

fn page_has_no_raw_bytes(projected: IntegrityValidatedInlineRecordProjection<'_, '_>) {
    let _: &[u8] = projected.bytes();
}

fn extent_has_no_raw_bytes(projected: IntegrityValidatedExtentChunkProjection<'_, '_>) {
    let _: &[u8] = projected.bytes();
}

fn wal_has_no_raw_bytes(projected: IntegrityValidatedWalPayloadProjection<'_, '_>) {
    let _: &[u8] = projected.bytes();
}

fn checkpoint_has_no_raw_bytes(
    projected: IntegrityValidatedCheckpointBindingPayloadProjection<'_, '_>,
) {
    let _: &[u8] = projected.bytes();
}

fn escape_page<'view, 'media>(
    projected: IntegrityValidatedInlineRecordProjection<'view, 'media>,
) -> IntegrityValidatedInlineRecordProjection<'static, 'media> {
    projected
}

fn escape_extent<'view, 'media>(
    projected: IntegrityValidatedExtentChunkProjection<'view, 'media>,
) -> IntegrityValidatedExtentChunkProjection<'static, 'media> {
    projected
}

fn escape_wal<'view, 'media>(
    projected: IntegrityValidatedWalPayloadProjection<'view, 'media>,
) -> IntegrityValidatedWalPayloadProjection<'static, 'media> {
    projected
}

fn escape_checkpoint<'view, 'media>(
    projected: IntegrityValidatedCheckpointBindingPayloadProjection<'view, 'media>,
) -> IntegrityValidatedCheckpointBindingPayloadProjection<'static, 'media> {
    projected
}

fn main() {}
