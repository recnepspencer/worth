pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::encode_inline_page, worth_store_physical_format::InlineRecordAppend, worth_store_physical_format::PageGenerationCell, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRecordSlot, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::SlotGenerationCell};
pub(crate) use {worth_store_physical_integrity::validate_inline_page, worth_store_physical_integrity::InlinePageIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) const PAGE_OFFSET: u64 = 1_048_576;
pub(crate) const PAGE_SIZES: [PhysicalPageSizeClass; 3] = [
    worth_store_physical_format::PhysicalPageSizeClass::KiB16,
    worth_store_physical_format::PhysicalPageSizeClass::KiB32,
    worth_store_physical_format::PhysicalPageSizeClass::KiB64,
];

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(crate) fn format(page_size: PhysicalPageSizeClass) -> PhysicalRecordFormatDeclaration {
    worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(page_size)
        .admit()
        .unwrap()
}

pub(crate) fn page(segment: u64, page: u64, generation: u64) -> PageGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            worth_store_physical_format::PhysicalSegmentId::from_raw(segment).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(page).unwrap(),
        )
        .with_page_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap())
}

pub(crate) fn slot(page: PageGenerationCell, slot: u16, generation: u64) -> SlotGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            page.segment_id(),
            page.page_id(),
            worth_store_physical_format::PhysicalRecordSlot::from_raw(slot).unwrap(),
        )
        .with_slot_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap())
}

pub(crate) fn record(byte: u8, ordinal: u64) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([byte; 16], ordinal).unwrap()
}

pub(crate) fn clean_page(page_size: PhysicalPageSizeClass, identity: PageGenerationCell) -> Vec<u8> {
    encode_inline_page(
        format(page_size),
        identity,
        &[
            worth_store_physical_format::InlineRecordAppend::new(record(0xa1, 1), slot(identity, 1, 21), b"alpha"),
            worth_store_physical_format::InlineRecordAppend::new(record(0xb2, 2), slot(identity, 2, 22), b"beta!!"),
        ],
    )
    .unwrap()
}

pub(crate) fn page_scope(
    store: StableStoreIdentity,
    page_size: PhysicalPageSizeClass,
    identity: PageGenerationCell,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::inline_page(
        store,
        format(page_size),
        identity,
        worth_store_physical_integrity::PhysicalByteRange::new(PAGE_OFFSET, u64::from(page_size.bytes())).unwrap(),
    )
}

pub(crate) fn validate_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let (validation, counters) =
        validate_inline_page(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let worth_store_physical_integrity::InlinePageIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged inline page unexpectedly validated");
    };
    (rejection, counters)
}

pub(crate) fn assert_damage(
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
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(worth_store_physical_integrity::PhysicalDamageLocalization::new(
            scope,
            cause,
            range,
            field,
            blast_radius,
        ))
    );
    assert_eq!(
        counters.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PageFrame
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}

pub(crate) fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub(crate) fn reseal(bytes: &mut [u8]) -> u32 {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
    checksum
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