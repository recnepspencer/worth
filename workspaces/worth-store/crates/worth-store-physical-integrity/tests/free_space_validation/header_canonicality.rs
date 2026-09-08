use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableFrameDenial, DurableFreeSpaceManifestHeader,
    FreeSpaceHeaderScopeIdentity, FreeSpaceRoutingDenial, PhysicalGeneration, PhysicalTreeIdentity,
};
use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, format, header_scope, independent_crc32c, range,
    reseal, store, HEADER_LITERAL, HEADER_OFFSET,
};

#[test]
fn resealed_metadata_page_lsn_is_denied_before_complete_child_admission() {
    assert!(matches!(
        validate_free_space_header(
            UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL),
            header_scope(store(7), independent_crc32c(&[HEADER_LITERAL])),
        )
        .0,
        FreeSpaceHeaderIntegrityValidation::Intact(_)
    ));
    for offset in 36..44 {
        let mut bytes = HEADER_LITERAL.to_vec();
        bytes[offset] = 1;
        reseal(&mut bytes);
        // Reseal the independently supplied parent binding too: stale CRC must
        // not be what prevents this noncanonical header from becoming a view.
        let scope = header_scope(store(7), independent_crc32c(&[&bytes]));
        assert!(matches!(
            DurableFreeSpaceManifestHeader::decode(&bytes, 2),
            Err(FreeSpaceRoutingDenial::Frame(
                DurableFrameDenial::NonDataPageLsnNonZero
            ))
        ));
        assert_canonicality_damage(&bytes, scope, 36, 8);
    }
}

#[test]
fn absent_root_requires_its_entire_reference_region_to_remain_zero() {
    let mut empty = HEADER_LITERAL.to_vec();
    empty[72..80].fill(0);
    empty[112..176].fill(0);
    reseal(&mut empty);
    let scope_for = |bytes: &[u8]| {
        PhysicalArtifactScope::free_space_header(
            store(7),
            format(),
            FreeSpaceHeaderScopeIdentity::new(
                PhysicalGeneration::from_raw(6).unwrap(),
                PhysicalTreeIdentity::new(8).unwrap(),
                None,
                DurableArtifactCrc32c::new(independent_crc32c(&[bytes])),
            ),
            PhysicalByteRange::new(HEADER_OFFSET, 176).unwrap(),
        )
    };
    assert!(DurableFreeSpaceManifestHeader::decode(&empty, 2).is_ok());
    assert!(matches!(
        validate_free_space_header(
            UntrustedPhysicalArtifact::from_bounded_bytes(&empty),
            scope_for(&empty),
        )
        .0,
        FreeSpaceHeaderIntegrityValidation::Intact(_)
    ));
    for offset in 120..176 {
        let mut bytes = empty.clone();
        bytes[offset] = 1;
        reseal(&mut bytes);
        assert!(matches!(
            DurableFreeSpaceManifestHeader::decode(&bytes, 2),
            Err(FreeSpaceRoutingDenial::Malformed)
        ));
        assert_canonicality_damage(&bytes, scope_for(&bytes), 120, 56);
    }
}

fn assert_canonicality_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    offset: u64,
    length: u64,
) {
    let (FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("noncanonical header admitted");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::MalformedStructure,
        range(scope, offset, length),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        176, PhysicalIntegrityRejectionClass::Damaged(PhysicalDamageCause::MalformedStructure));
}
