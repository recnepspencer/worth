use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurableArtifactCrc32c, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalGeneration, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, PhysicalTreeIdentity, RecordAllocationClass,
};
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField, PhysicalIntegrityObservationCounters,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
};

pub const HEADER_OFFSET: u64 = 4_096;
pub const MEMBERSHIP_OFFSET: u64 = 8_192;
pub const HEADER_COMPLETE_CRC32C: u32 = 0xe17d_629e;
pub const MEMBERSHIP_COMPLETE_CRC32C: u32 = 0x1ff2_a0de;

pub const MEMBERSHIP_LITERAL: &[u8; 168] = &[
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x0a, 0x02, 0x01, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x25, 0x2a, 0xb6, 0x8b,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub const HEADER_LITERAL: &[u8; 176] = &[
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x07, 0x02, 0x01, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xca, 0xb4, 0x80, 0x31,
    0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xde, 0xa0, 0xf2, 0x1f,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB16)
        .admit()
        .unwrap()
}

pub fn first_key() -> FreeSpaceKey {
    FreeSpaceKey::new(RecordAllocationClass::InlinePage, 7).unwrap()
}

pub fn last_key() -> FreeSpaceKey {
    FreeSpaceKey::new(RecordAllocationClass::Extent, 5).unwrap()
}

pub fn membership_reference(checksum: u32) -> FreeSpaceBlockReference {
    FreeSpaceBlockReference::new(6, 1, 0, checksum, first_key(), last_key()).unwrap()
}

pub fn membership_scope(
    store: StableStoreIdentity,
    reference: FreeSpaceBlockReference,
) -> PhysicalArtifactScope {
    membership_scope_at(
        store,
        reference,
        PhysicalByteRange::new(MEMBERSHIP_OFFSET, MEMBERSHIP_LITERAL.len() as u64).unwrap(),
    )
}

pub fn membership_scope_at(
    store: StableStoreIdentity,
    reference: FreeSpaceBlockReference,
    range: PhysicalByteRange,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::free_space_membership_block(
        store,
        format(),
        FreeSpaceMembershipBlockScopeIdentity::new(
            PhysicalTreeIdentity::new(8).unwrap(),
            reference,
        ),
        range,
    )
}

pub fn header_scope(store: StableStoreIdentity, checksum: u32) -> PhysicalArtifactScope {
    header_scope_at(
        store,
        checksum,
        PhysicalByteRange::new(HEADER_OFFSET, HEADER_LITERAL.len() as u64).unwrap(),
    )
}

pub fn header_scope_at(
    store: StableStoreIdentity,
    checksum: u32,
    range: PhysicalByteRange,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::free_space_header(
        store,
        format(),
        FreeSpaceHeaderScopeIdentity::new(
            PhysicalGeneration::from_raw(6).unwrap(),
            PhysicalTreeIdentity::new(8).unwrap(),
            Some(membership_reference(MEMBERSHIP_COMPLETE_CRC32C)),
            DurableArtifactCrc32c::new(checksum),
        ),
        range,
    )
}

pub fn reseal(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

pub fn range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            cause,
            damaged_range,
            field,
            blast_radius,
        ))
    );
}

pub fn assert_intact_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    byte_count: u64,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}

pub fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    byte_count: u64,
    class: PhysicalIntegrityRejectionClass,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(counters.rejected_for(class), 1);
}
