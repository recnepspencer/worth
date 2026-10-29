pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::FreeSpaceBlockReference, worth_store_physical_format::FreeSpaceKey, worth_store_physical_format::ManifestBlockReference, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::RootSelectorIdentity, worth_store_physical_format::RootSelectorRole, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::SegmentPageKey, worth_store_physical_format::ROOT_SELECTOR_BYTES};
pub(crate) use {worth_store_physical_integrity::validate_current_root_selector, worth_store_physical_integrity::validate_previous_root_selector, worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) const SELECTOR_OFFSET: u64 = 4_096;
pub(crate) const MANIFEST_OFFSET: u64 = 16_384;
pub(crate) const MANIFEST_BYTES: u64 = 384;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SelectorKind {
    Current,
    Previous,
}

impl SelectorKind {
    pub(crate) const ALL: [Self; 2] = [Self::Current, Self::Previous];

    pub(crate) const fn role(self) -> RootSelectorRole {
        match self {
            Self::Current => worth_store_physical_format::RootSelectorRole::Current,
            Self::Previous => worth_store_physical_format::RootSelectorRole::Previous,
        }
    }

    pub(crate) const fn family(self) -> PhysicalIntegrityArtifactFamily {
        match self {
            Self::Current => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            Self::Previous => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        }
    }

    pub(crate) const fn opposite(self) -> Self {
        match self {
            Self::Current => Self::Previous,
            Self::Previous => Self::Current,
        }
    }
}

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

pub(crate) fn selector_bytes(
    kind: SelectorKind,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> [u8; ROOT_SELECTOR_BYTES] {
    let (identity, root_generation, linked_identity, linked_generation) = match kind {
        crate::support::root_validation::SelectorKind::Current => (101, 11, 99, 10),
        crate::support::root_validation::SelectorKind::Previous => (99, 10, 97, 9),
    };
    worth_store_physical_format::DurableRootSelector::new(
        store,
        format,
        worth_store_physical_format::RootSelectorIdentity::new(identity).unwrap(),
        kind.role(),
        root_generation,
        worth_store_physical_format::RootSelectorIdentity::new(linked_identity),
        Some(linked_generation),
    )
    .unwrap()
    .encode()
}

pub(crate) fn selector_scope(
    kind: SelectorKind,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> PhysicalArtifactScope {
    let range = worth_store_physical_integrity::PhysicalByteRange::new(SELECTOR_OFFSET, ROOT_SELECTOR_BYTES as u64).unwrap();
    match kind {
        crate::support::root_validation::SelectorKind::Current => worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(store, format, range),
        crate::support::root_validation::SelectorKind::Previous => {
            worth_store_physical_integrity::PhysicalArtifactScope::previous_root_selector(store, format, range)
        }
    }
}

pub(crate) fn validate_selector_rejection(
    kind: SelectorKind,
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    match kind {
        crate::support::root_validation::SelectorKind::Current => match validate_current_root_selector(artifact, scope) {
            (worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            (worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation::Intact(_), _) => {
                panic!("current selector unexpectedly validated")
            }
        },
        crate::support::root_validation::SelectorKind::Previous => match validate_previous_root_selector(artifact, scope) {
            (worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            (worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation::Intact(_), _) => {
                panic!("previous selector unexpectedly validated")
            }
        },
    }
}

pub(crate) fn manifest_bytes(generation: u64, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
    manifest_bytes_with_capacity(generation, format, 2)
}

pub(crate) fn manifest_bytes_with_capacity(
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
) -> Vec<u8> {
    let key = worth_store_physical_format::FreeSpaceKey::inline(1).unwrap();
    let free_space_root = worth_store_physical_format::FreeSpaceBlockReference::new(generation, 1, 0, 41, key, key).unwrap();
    worth_store_physical_format::DurablePhysicalRootManifest::builder(generation, 71, node_capacity, 43)
        .free_space_root(Some(free_space_root))
        .admit()
        .unwrap()
        .encode(format)
}

pub(crate) fn populated_manifest_bytes(
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Vec<u8> {
    let record = worth_store_physical_format::PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    let routing_root = worth_store_physical_format::ManifestBlockReference::new(generation, 1, 0, 51, record, record).unwrap();
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap();
    let segment_key = worth_store_physical_format::SegmentPageKey::new(segment, worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap());
    let segment_root =
        worth_store_physical_format::SegmentManifestBlockReference::new(generation, 1, 0, 52, segment_key, segment_key).unwrap();
    let segment_cell = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .segment_cell(segment)
        .with_segment_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap());
    let free_key = worth_store_physical_format::FreeSpaceKey::inline(1).unwrap();
    let free_space_root =
        worth_store_physical_format::FreeSpaceBlockReference::new(generation, 1, 0, 41, free_key, free_key).unwrap();
    worth_store_physical_format::DurablePhysicalRootManifest::builder(generation, 71, 2, 43)
        .record_count(1)
        .next_block(2)
        .next_segment_block(2)
        .routing_root(Some(routing_root))
        .segment_root(Some(segment_root))
        .free_space_root(Some(free_space_root))
        .last_inline_record(Some(record))
        .last_inline_segment(Some(segment_cell))
        .admit()
        .unwrap()
        .encode(format)
}

pub(crate) fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        worth_store_physical_integrity::PhysicalByteRange::new(MANIFEST_OFFSET, MANIFEST_BYTES).unwrap(),
    )
    .unwrap()
}

pub(crate) fn reseal_durable_frame(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
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
}

pub(crate) fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    byte_count: u64,
    cause: PhysicalDamageCause,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
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