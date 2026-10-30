pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    encode_extent_chunk, ExtentChunkCoordinate, PhysicalPageSizeClass,
};
pub(crate) use worth_store_physical_integrity::{
    validate_extent_chunk, ExtentChunkIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::extent_validation::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, chunk_scope, extent_cell,
    field_range, format, record, reseal_durable_frame, store, validated_manifest, ExtentFixture,
    CHUNK_OFFSET,
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

pub(crate) fn assert_chunk_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (validation, counters) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    );
    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("damaged extent chunk unexpectedly validated")
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk,
        bytes.len() as u64,
        cause,
    );
}

pub(crate) fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (
        worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported),
        ),
        counters,
    ) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    )
    else {
        panic!("expected unsupported extent chunk version")
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.rejected_for(
            worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported
        ),
        1
    );
}
