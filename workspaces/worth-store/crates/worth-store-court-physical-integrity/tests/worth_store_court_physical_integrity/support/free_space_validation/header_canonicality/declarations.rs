pub(crate) use {worth_store_physical_format::DurableArtifactCrc32c, worth_store_physical_format::DurableFrameDenial, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::FreeSpaceHeaderScopeIdentity, worth_store_physical_format::FreeSpaceRoutingDenial, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalTreeIdentity};
pub(crate) use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::format, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::independent_crc32c, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_LITERAL, crate::support::free_space_validation::HEADER_OFFSET};

pub(crate) fn assert_canonicality_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    offset: u64,
    length: u64,
) {
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("noncanonical header admitted");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        range(scope, offset, length),
        Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        HEADER_LITERAL.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure));
}