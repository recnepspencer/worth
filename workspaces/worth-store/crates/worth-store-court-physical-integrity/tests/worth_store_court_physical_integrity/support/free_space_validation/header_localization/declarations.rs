pub(crate) use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_COMPLETE_CRC32C, crate::support::free_space_validation::HEADER_LITERAL};

pub(crate) fn assert_header_localization(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
) {
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("damaged header validated");
    };
    assert_damage(
        rejection,
        scope,
        cause,
        range(scope, offset, length),
        Some(field),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}