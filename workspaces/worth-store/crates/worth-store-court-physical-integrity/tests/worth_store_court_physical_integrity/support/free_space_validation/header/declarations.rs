pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
pub(crate) use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_intact_counters, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::format, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::header_scope_at, crate::support::free_space_validation::independent_crc32c, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_COMPLETE_CRC32C, crate::support::free_space_validation::HEADER_LITERAL, crate::support::free_space_validation::HEADER_OFFSET};

pub(crate) fn assert_header_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("unsupported header validated");
    };
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) = rejection else {
        panic!("unsupported header collapsed into damage");
    };
    assert_eq!(version.axis(), axis);
    assert_eq!(version.observed(), observed);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported,
    );
}

pub(crate) fn assert_header_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("damaged free-space header validated");
    };
    assert_damage(rejection, scope, cause, damaged_range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}