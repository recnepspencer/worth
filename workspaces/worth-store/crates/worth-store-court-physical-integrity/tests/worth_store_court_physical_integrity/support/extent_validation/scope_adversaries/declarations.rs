pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    encode_extent_chunk, ExtentChunkCoordinate, PhysicalPageSizeClass,
};
pub(crate) use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalDamageCause, PhysicalFormatField, UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::extent_validation::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, extent_cell, field_range,
    format, record, reseal_durable_frame, validated_manifest, ExtentFixture,
};

pub(crate) fn coordinate(
    record: worth_store_physical_format::PersistedRecordIdentity,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_bytes: u64,
    logical_offset: u64,
    ordinal: u32,
) -> ExtentChunkCoordinate {
    worth_store_physical_format::ExtentChunkCoordinate::new(
        record,
        extent,
        logical_bytes,
        logical_offset,
        ordinal,
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn assert_chunk_rejected(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) {
    let (
        worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(rejection),
        counters,
    ) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    )
    else {
        panic!("substituted extent chunk unexpectedly validated")
    };
    assert_damage(
        rejection,
        scope,
        cause,
        field_range(scope, offset, length),
        Some(field),
        blast_radius,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk,
        bytes.len() as u64,
        cause,
    );
}
