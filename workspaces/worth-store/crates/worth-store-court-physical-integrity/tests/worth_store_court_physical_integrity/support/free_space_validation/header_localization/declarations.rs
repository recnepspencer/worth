pub(crate) use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalBlastRadius,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejectionClass,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::free_space_validation::{
    assert_damage, assert_rejected_counters, header_scope, range, reseal, store,
    HEADER_COMPLETE_CRC32C, HEADER_LITERAL,
};

pub(crate) fn assert_header_localization(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
) {
    let (
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection),
        counters,
    ) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
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
