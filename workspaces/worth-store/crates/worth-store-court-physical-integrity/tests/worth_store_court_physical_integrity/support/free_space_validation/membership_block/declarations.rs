pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::ExtentArenaId, worth_store_physical_format::FreeSpaceBlockReference, worth_store_physical_format::FreeSpaceKey, worth_store_physical_format::RecordAllocationClass};
pub(crate) use {worth_store_physical_integrity::validate_free_space_membership_block, worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::free_space_validation::assert_intact_counters, crate::support::free_space_validation::assert_membership_damage, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::first_key, crate::support::free_space_validation::format, crate::support::free_space_validation::independent_crc32c, crate::support::free_space_validation::last_key, crate::support::free_space_validation::membership_reference, crate::support::free_space_validation::membership_scope, crate::support::free_space_validation::membership_scope_at, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::MEMBERSHIP_COMPLETE_CRC32C, crate::support::free_space_validation::MEMBERSHIP_LITERAL, crate::support::free_space_validation::MEMBERSHIP_OFFSET};

pub(crate) fn assert_membership_unsupported(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_membership_block(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        )
    else {
        panic!("unsupported membership block validated");
    };
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) =
        rejection
    else {
        panic!("unsupported format collapsed into damage");
    };
    assert_eq!(version.axis(), axis);
    assert_eq!(version.observed(), observed);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported,
    );
}