#[allow(unused_imports)] use crate::free_space_validation::header_canonicality::*;
use {worth_store_physical_format::DurableArtifactCrc32c, worth_store_physical_format::DurableFrameDenial, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::FreeSpaceHeaderScopeIdentity, worth_store_physical_format::FreeSpaceRoutingDenial, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalTreeIdentity};
use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::format, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::independent_crc32c, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_LITERAL, crate::support::free_space_validation::HEADER_OFFSET};

#[test]
fn resealed_metadata_page_lsn_is_denied_before_complete_child_admission() {
    assert!(matches!(
        validate_free_space_header(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL),
            header_scope(store(7), independent_crc32c(&[HEADER_LITERAL])),
        )
        .0,
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(_)
    ));
    for offset in 36..44 {
        let mut bytes = HEADER_LITERAL.to_vec();
        bytes[offset] = 1;
        reseal(&mut bytes);
        // Reseal the independently supplied parent binding too: stale CRC must
        // not be what prevents this noncanonical header from becoming a view.
        let scope = header_scope(store(7), independent_crc32c(&[&bytes]));
        assert!(matches!(
            worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&bytes, 2),
            Err(worth_store_physical_format::FreeSpaceRoutingDenial::Frame(
                worth_store_physical_format::DurableFrameDenial::NonDataPageLsnNonZero
            ))
        ));
        assert_canonicality_damage(&bytes, scope, 36, 8);
    }
}

#[test]
fn absent_root_requires_its_entire_reference_region_to_remain_zero() {
    let mut empty = HEADER_LITERAL.to_vec();
    empty[72..80].fill(0);
    empty[112..192].fill(0);
    reseal(&mut empty);
    let scope_for = |bytes: &[u8]| {
        worth_store_physical_integrity::PhysicalArtifactScope::free_space_header(
            store(7),
            format(),
            worth_store_physical_format::FreeSpaceHeaderScopeIdentity::new(
                worth_store_physical_format::PhysicalGeneration::from_raw(6).unwrap(),
                worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
                None,
                worth_store_physical_format::DurableArtifactCrc32c::new(independent_crc32c(&[bytes])),
            ),
            worth_store_physical_integrity::PhysicalByteRange::new(HEADER_OFFSET, HEADER_LITERAL.len() as u64).unwrap(),
        )
    };
    assert!(worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&empty, 2).is_ok());
    assert!(matches!(
        validate_free_space_header(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&empty),
            scope_for(&empty),
        )
        .0,
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(_)
    ));
    for offset in 120..192 {
        let mut bytes = empty.clone();
        bytes[offset] = 1;
        reseal(&mut bytes);
        assert!(matches!(
            worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&bytes, 2),
            Err(worth_store_physical_format::FreeSpaceRoutingDenial::Malformed)
        ));
        assert_canonicality_damage(&bytes, scope_for(&bytes), 120, 72);
    }
}

#[test]
fn arena_frontier_and_geometry_fail_at_their_exact_fields() {
    let mut missing_frontier = HEADER_LITERAL.to_vec();
    missing_frontier[192..200].fill(0);
    reseal(&mut missing_frontier);
    let scope = header_scope(store(7), independent_crc32c(&[&missing_frontier]));
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), _) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&missing_frontier),
        scope,
    ) else {
        panic!("zero arena frontier admitted");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        range(scope, 192, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::AllocationFrontier),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut invalid_alignment = HEADER_LITERAL.to_vec();
    invalid_alignment[208..216].copy_from_slice(&3_u64.to_le_bytes());
    reseal(&mut invalid_alignment);
    let scope = header_scope(store(7), independent_crc32c(&[&invalid_alignment]));
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection), _) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&invalid_alignment),
        scope,
    ) else {
        panic!("non-power-of-two arena alignment admitted");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        range(scope, 208, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::Payload),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
