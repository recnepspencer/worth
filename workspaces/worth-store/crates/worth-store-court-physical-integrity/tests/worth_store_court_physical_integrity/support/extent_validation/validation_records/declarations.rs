pub(crate) use {worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalIntegrityValidationRecord, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::extent_validation::chunk_payload_capacity, crate::support::extent_validation::chunk_scope, crate::support::extent_validation::extent_cell, crate::support::extent_validation::format, crate::support::extent_validation::record, crate::support::extent_validation::store, crate::support::extent_validation::validated_manifest, crate::support::extent_validation::ExtentFixture};

pub(crate) fn records(
    fixture: ExtentFixture,
    ordinal: u32,
) -> (
    PhysicalIntegrityValidationRecord,
    PhysicalIntegrityValidationRecord,
) {
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let manifest_record =
        validated_manifest(&manifest_bytes, fixture.manifest_scope()).into_validation_record();
    let coordinate = fixture.chunk_coordinate(ordinal);
    let length = if ordinal == 1 {
        chunk_payload_capacity(fixture.format)
    } else {
        fixture.logical_bytes - chunk_payload_capacity(fixture.format)
    };
    let payload = vec![0xA5; usize::try_from(length).unwrap()];
    let chunk_bytes = encode_extent_chunk(fixture.format, coordinate, &payload).unwrap();
    let scope = chunk_scope(
        fixture.store,
        fixture.format,
        coordinate,
        chunk_bytes.len() as u64,
    );
    (
        manifest_record,
        chunk_record(&chunk_bytes, scope, &manifest),
    )
}

pub(crate) fn chunk_record(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
) -> PhysicalIntegrityValidationRecord {
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(chunk), _) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    ) else {
        panic!("clean extent chunk rejected")
    };
    chunk.into_validation_record()
}