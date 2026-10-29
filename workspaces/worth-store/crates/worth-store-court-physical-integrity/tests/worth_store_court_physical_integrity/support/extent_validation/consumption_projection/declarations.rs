pub(crate) use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageLsn};
pub(crate) use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::validate_extent_chunk_membership, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::ExtentChunkProjectionDenial, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::SelectedExtentPayloadBuilder, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::extent_validation::chunk_payload_capacity, crate::support::extent_validation::chunk_scope, crate::support::extent_validation::manifest_scope, crate::support::extent_validation::record, crate::support::extent_validation::store, crate::support::extent_validation::validated_manifest, crate::support::extent_validation::ExtentFixture};

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
        .record_extent_cell(worth_store_physical_format::PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap())
}