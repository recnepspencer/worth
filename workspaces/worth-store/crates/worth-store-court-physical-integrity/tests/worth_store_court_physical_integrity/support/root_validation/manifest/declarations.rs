pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::maximum_current_root_entries, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_root_manifest, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::RootManifestIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::root_validation::assert_damage, crate::support::root_validation::assert_rejected_counters, crate::support::root_validation::field_range, crate::support::root_validation::format, crate::support::root_validation::manifest_bytes, crate::support::root_validation::manifest_bytes_with_capacity, crate::support::root_validation::manifest_scope, crate::support::root_validation::populated_manifest_bytes, crate::support::root_validation::reseal_durable_frame, crate::support::root_validation::selector_scope, crate::support::root_validation::store, crate::support::root_validation::SelectorKind, crate::support::root_validation::MANIFEST_BYTES, crate::support::root_validation::MANIFEST_OFFSET};

pub(crate) fn assert_manifest_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (validation, counters) =
        validate_root_manifest(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let worth_store_physical_integrity::RootManifestIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged root manifest unexpectedly validated");
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest,
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
    let (validation, counters) =
        validate_root_manifest(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let worth_store_physical_integrity::RootManifestIntegrityValidation::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(
        unsupported,
    )) = validation
    else {
        panic!("expected unsupported manifest version");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

pub(crate) fn assert_intact_counters(
    counters: worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
    byte_count: u64,
) {
    assert_eq!(
        counters.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}