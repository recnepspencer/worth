pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::DurableArtifactCrc32c, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::FreeSpaceBlockReference, worth_store_physical_format::FreeSpaceHeaderScopeIdentity, worth_store_physical_format::FreeSpaceKey, worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalTreeIdentity};
pub(crate) use {worth_store_physical_integrity::validate_free_space_membership_block, worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) const HEADER_OFFSET: u64 = 4_096;
pub(crate) const MEMBERSHIP_OFFSET: u64 = 8_192;
pub(crate) const HEADER_COMPLETE_CRC32C: u32 = 0x6ff0_c09c;
pub(crate) const MEMBERSHIP_COMPLETE_CRC32C: u32 = 0xa980_5406;

pub(crate) const MEMBERSHIP_LITERAL: &[u8; 168] = &[
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x0a, 0x02, 0x02, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb8, 0x22, 0xdc, 0xf1,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub(crate) const HEADER_LITERAL: &[u8; 216] = &[
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x07, 0x02, 0x02, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0xa8, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x90, 0x1c, 0x2d,
    0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x54, 0x80, 0xa9,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(crate) fn format() -> PhysicalRecordFormatDeclaration {
    worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB16)
        .admit()
        .unwrap()
}

pub(crate) fn first_key() -> FreeSpaceKey {
    worth_store_physical_format::FreeSpaceKey::inline(7).unwrap()
}

pub(crate) fn last_key() -> FreeSpaceKey {
    worth_store_physical_format::FreeSpaceKey::arena(worth_store_physical_format::ExtentArenaId::new(5).unwrap(), 100)
}

pub(crate) fn membership_reference(checksum: u32) -> FreeSpaceBlockReference {
    worth_store_physical_format::FreeSpaceBlockReference::new(6, 1, 0, checksum, first_key(), last_key()).unwrap()
}

pub(crate) fn membership_scope(
    store: StableStoreIdentity,
    reference: FreeSpaceBlockReference,
) -> PhysicalArtifactScope {
    membership_scope_at(
        store,
        reference,
        worth_store_physical_integrity::PhysicalByteRange::new(MEMBERSHIP_OFFSET, MEMBERSHIP_LITERAL.len() as u64).unwrap(),
    )
}

pub(crate) fn membership_scope_at(
    store: StableStoreIdentity,
    reference: FreeSpaceBlockReference,
    range: PhysicalByteRange,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
        store,
        format(),
        worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity::new(
            worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
            reference,
        ),
        range,
    )
}

pub(crate) fn header_scope(store: StableStoreIdentity, checksum: u32) -> PhysicalArtifactScope {
    header_scope_at(
        store,
        checksum,
        worth_store_physical_integrity::PhysicalByteRange::new(HEADER_OFFSET, HEADER_LITERAL.len() as u64).unwrap(),
    )
}

pub(crate) fn header_scope_at(
    store: StableStoreIdentity,
    checksum: u32,
    range: PhysicalByteRange,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::free_space_header(
        store,
        format(),
        worth_store_physical_format::FreeSpaceHeaderScopeIdentity::new(
            worth_store_physical_format::PhysicalGeneration::from_raw(6).unwrap(),
            worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
            Some(membership_reference(MEMBERSHIP_COMPLETE_CRC32C)),
            worth_store_physical_format::DurableArtifactCrc32c::new(checksum),
        ),
        range,
    )
}

pub(crate) fn reseal(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

pub(crate) fn range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub(crate) fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(worth_store_physical_integrity::PhysicalDamageLocalization::new(
            scope,
            cause,
            damaged_range,
            field,
            blast_radius,
        ))
    );
}

pub(crate) fn assert_intact_counters(
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

pub(crate) fn assert_rejected_counters(
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

pub(crate) fn assert_membership_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_membership_block(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        )
    else {
        panic!("damaged free-space membership block validated");
    };
    assert_damage(rejection, scope, cause, damaged_range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}