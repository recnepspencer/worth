#[allow(unused_imports)] use crate::free_space_validation::header_localization::*;
use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_COMPLETE_CRC32C, crate::support::free_space_validation::HEADER_LITERAL};

#[test]
fn header_envelope_payload_and_zero_identities_blame_the_substituted_field() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    for (offset, value, cause, field) in [
        (
            28,
            7_u64,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            7,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            0,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            56,
            0,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::TreeIdentity,
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
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        28,
        8,
        worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
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
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        112,
        80,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
    );

    let mut missing_count = HEADER_LITERAL.to_vec();
    missing_count[72..80].fill(0);
    reseal(&mut missing_count);
    assert_header_localization(
        &missing_count,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        72,
        8,
        worth_store_physical_integrity::PhysicalFormatField::FreeSpaceEntryCount,
    );
}
