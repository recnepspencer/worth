pub(crate) use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableFrameDenial, DurableFreeSpaceManifestHeader,
    FreeSpaceHeaderScopeIdentity, FreeSpaceRoutingDenial, PhysicalGeneration, PhysicalTreeIdentity,
};
pub(crate) use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::free_space_validation::{
    assert_damage, assert_rejected_counters, format, header_scope, independent_crc32c, range,
    reseal, store, HEADER_LITERAL, HEADER_OFFSET,
};

pub(crate) fn assert_canonicality_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    offset: u64,
    length: u64,
) {
    let (
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection),
        counters,
    ) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
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
