use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalBlastRadius,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejectionClass,
    UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, header_scope, range, reseal, store,
    HEADER_COMPLETE_CRC32C, HEADER_LITERAL,
};

#[test]
fn header_envelope_payload_and_zero_identities_blame_the_substituted_field() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    for (offset, value, cause, field) in [
        (
            28,
            7_u64,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            7,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            0,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            56,
            0,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalFormatField::TreeIdentity,
        ),
    ] {
        let mut bytes = HEADER_LITERAL.to_vec();
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        reseal(&mut bytes);
        assert_header_localization(&bytes, scope, cause, offset as u64, 8, field);
    }

    let mut coherent = HEADER_LITERAL.to_vec();
    coherent[28..36].copy_from_slice(&7_u64.to_le_bytes());
    coherent[48..56].copy_from_slice(&7_u64.to_le_bytes());
    reseal(&mut coherent);
    assert_header_localization(
        &coherent,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        28,
        8,
        PhysicalFormatField::PhysicalGeneration,
    );
}

#[test]
fn header_root_presence_and_entry_count_shape_failures_are_distinct() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    let mut missing_root_presence = HEADER_LITERAL.to_vec();
    missing_root_presence[112] = 0;
    reseal(&mut missing_root_presence);
    assert_header_localization(
        &missing_root_presence,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        112,
        64,
        PhysicalFormatField::ChildReference,
    );

    let mut missing_count = HEADER_LITERAL.to_vec();
    missing_count[72..80].fill(0);
    reseal(&mut missing_count);
    assert_header_localization(
        &missing_count,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        72,
        8,
        PhysicalFormatField::FreeSpaceEntryCount,
    );
}

fn assert_header_localization(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
) {
    let (FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("damaged header validated");
    };
    assert_damage(
        rejection,
        scope,
        cause,
        range(scope, offset, length),
        Some(field),
        PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}
