pub(crate) use worth_store_physical_format::{
    encode_data_frame_page_lsn, encode_extent_chunk, DurableExtentRecordPlacement,
    DurableFrameKind, ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageLsn,
};
pub(crate) use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_chunk_membership, ExtentChunkIntegrityValidation,
    ExtentChunkProjectionDenial, PhysicalArtifactScope, PhysicalDamageCause,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass, SelectedExtentPayloadBuilder,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::extent_validation::{
    chunk_payload_capacity, chunk_scope, manifest_scope, record, store, validated_manifest,
    ExtentFixture,
};

pub(crate) fn coordinate(
    fixture: ExtentFixture,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_offset: u64,
    ordinal: u32,
) -> ExtentChunkCoordinate {
    worth_store_physical_format::ExtentChunkCoordinate::new(
        fixture.record,
        extent,
        fixture.logical_bytes,
        logical_offset,
        ordinal,
    )
    .unwrap()
}

pub(crate) fn extent_cell(
    extent: u64,
    generation: u64,
) -> worth_store_physical_format::RecordExtentGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(
            worth_store_physical_format::PhysicalExtentId::from_raw(extent).unwrap(),
        )
        .with_extent_generation(
            worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap(),
        )
}
