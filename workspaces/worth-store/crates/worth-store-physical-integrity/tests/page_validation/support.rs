use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    encode_inline_page, InlineRecordAppend, PageGenerationCell, PersistedRecordIdentity,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, PhysicalRecordSlot, PhysicalSegmentId, SlotGenerationCell,
};
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField, PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

pub const PAGE_OFFSET: u64 = 1_048_576;
pub const PAGE_SIZES: [PhysicalPageSizeClass; 3] = [
    PhysicalPageSizeClass::KiB16,
    PhysicalPageSizeClass::KiB32,
    PhysicalPageSizeClass::KiB64,
];

pub fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub fn format(page_size: PhysicalPageSizeClass) -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .page_size(page_size)
        .admit()
        .unwrap()
}

pub fn page(segment: u64, page: u64, generation: u64) -> PageGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            PhysicalSegmentId::from_raw(segment).unwrap(),
            PhysicalPageId::from_raw(page).unwrap(),
        )
        .with_page_generation(PhysicalGeneration::from_raw(generation).unwrap())
}

pub fn slot(page: PageGenerationCell, slot: u16, generation: u64) -> SlotGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            page.segment_id(),
            page.page_id(),
            PhysicalRecordSlot::from_raw(slot).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(generation).unwrap())
}

pub fn record(byte: u8, ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([byte; 16], ordinal).unwrap()
}

pub fn clean_page(page_size: PhysicalPageSizeClass, identity: PageGenerationCell) -> Vec<u8> {
    encode_inline_page(
        format(page_size),
        identity,
        &[
            InlineRecordAppend::new(record(0xa1, 1), slot(identity, 1, 21), b"alpha"),
            InlineRecordAppend::new(record(0xb2, 2), slot(identity, 2, 22), b"beta!!"),
        ],
    )
    .unwrap()
}

pub fn page_scope(
    store: StableStoreIdentity,
    page_size: PhysicalPageSizeClass,
    identity: PageGenerationCell,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::inline_page(
        store,
        format(page_size),
        identity,
        PhysicalByteRange::new(PAGE_OFFSET, u64::from(page_size.bytes())).unwrap(),
    )
}

pub fn validate_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let (validation, counters) =
        validate_inline_page(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let InlinePageIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged inline page unexpectedly validated");
    };
    (rejection, counters)
}

pub fn assert_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_rejection(bytes, scope);
    assert_eq!(
        rejection,
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            cause,
            range,
            field,
            blast_radius,
        ))
    );
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::PageFrame
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}

pub fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub fn reseal(bytes: &mut [u8]) -> u32 {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
    checksum
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
