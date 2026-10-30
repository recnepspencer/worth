pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    ExtentArenaId, FreeSpaceBlockReference, FreeSpaceKey, RecordAllocationClass,
};
pub(crate) use worth_store_physical_integrity::{
    validate_free_space_membership_block, FreeSpaceMembershipBlockIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::free_space_validation::{
    assert_intact_counters, assert_membership_damage, assert_rejected_counters, first_key, format,
    independent_crc32c, last_key, membership_reference, membership_scope, membership_scope_at,
    range, reseal, store, MEMBERSHIP_COMPLETE_CRC32C, MEMBERSHIP_LITERAL, MEMBERSHIP_OFFSET,
};

pub(crate) fn assert_membership_unsupported(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Rejected(
            rejection,
        ),
        counters,
    ) = validate_free_space_membership_block(
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
