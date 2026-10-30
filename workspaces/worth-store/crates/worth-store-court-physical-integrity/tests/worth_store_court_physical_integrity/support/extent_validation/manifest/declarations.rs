pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{DurableExtentManifest, PhysicalPageSizeClass};
pub(crate) use worth_store_physical_integrity::{
    validate_extent_manifest, ExtentManifestIntegrityValidation, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::extent_validation::{
    assert_damage, assert_rejected_counters, chunk_payload_capacity, extent_cell, field_range,
    format, manifest_scope, record, reseal_durable_frame, store, ExtentFixture, MANIFEST_OFFSET,
};

pub(crate) fn manifest_bytes(
    fixture: ExtentFixture,
    record: worth_store_physical_format::PersistedRecordIdentity,
    extent: worth_store_physical_format::RecordExtentGenerationCell,
    logical_bytes: u64,
) -> Vec<u8> {
    worth_store_physical_format::DurableExtentManifest::new(
        fixture.format,
        record,
        extent,
        logical_bytes,
        fixture.format.page_size().bytes(),
        2,
        u64::from(fixture.format.page_size().bytes()),
    )
    .unwrap()
    .encode(fixture.format)
}

pub(crate) fn scope_with_length(
    fixture: ExtentFixture,
    length: u64,
) -> worth_store_physical_integrity::PhysicalArtifactScope {
    manifest_scope(fixture.store, fixture.format, fixture.placement(), length)
}

pub(crate) fn intact<'a>(
    bytes: &'a [u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
) -> worth_store_physical_integrity::IntegrityValidatedExtentManifest<'a> {
    let (worth_store_physical_integrity::ExtentManifestIntegrityValidation::Intact(validated), _) =
        validate_extent_manifest(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        )
    else {
        panic!("expected intact manifest")
    };
    validated
}

pub(crate) fn assert_manifest_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (validation, counters) = validate_extent_manifest(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let worth_store_physical_integrity::ExtentManifestIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("damaged extent manifest unexpectedly validated")
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest,
        bytes.len() as u64,
        cause,
    );
}

pub(crate) fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (
        worth_store_physical_integrity::ExtentManifestIntegrityValidation::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported),
        ),
        counters,
    ) = validate_extent_manifest(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    else {
        panic!("expected unsupported extent manifest version")
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
