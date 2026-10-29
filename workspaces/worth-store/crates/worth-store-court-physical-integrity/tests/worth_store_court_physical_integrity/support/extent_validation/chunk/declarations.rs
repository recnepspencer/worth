pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::extent_validation::assert_damage, crate::support::extent_validation::assert_rejected_counters, crate::support::extent_validation::chunk_payload_capacity, crate::support::extent_validation::chunk_scope, crate::support::extent_validation::extent_cell, crate::support::extent_validation::field_range, crate::support::extent_validation::format, crate::support::extent_validation::record, crate::support::extent_validation::reseal_durable_frame, crate::support::extent_validation::store, crate::support::extent_validation::validated_manifest, crate::support::extent_validation::ExtentFixture, crate::support::extent_validation::CHUNK_OFFSET};

pub(crate) fn coordinate(
    record: worth_store_physical_format::PersistedRecordIdentity,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_bytes: u64,
    logical_offset: u64,
    ordinal: u32,
) -> ExtentChunkCoordinate {
    worth_store_physical_format::ExtentChunkCoordinate::new(record, extent, logical_bytes, logical_offset, ordinal).unwrap()
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
    let worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(rejection) = validation else {
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
        worth_store_physical_integrity::ExtentChunkIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(
            unsupported,
        )),
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
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}