use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, FreeSpaceBlockReference, FreeSpaceKey,
    ManifestBlockReference, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, PhysicalSegmentId, RecordAllocationClass,
    RootSelectorIdentity, RootSelectorRole, SegmentManifestBlockReference, SegmentPageKey,
    ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{
    validate_current_root_selector, validate_previous_root_selector,
    CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization, PhysicalFormatField,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PreviousRootSelectorIntegrityValidation,
    UntrustedPhysicalArtifact,
};

pub const SELECTOR_OFFSET: u64 = 4_096;
pub const MANIFEST_OFFSET: u64 = 16_384;
pub const MANIFEST_BYTES: u64 = 368;

#[derive(Debug, Clone, Copy)]
pub enum SelectorKind {
    Current,
    Previous,
}

impl SelectorKind {
    pub const ALL: [Self; 2] = [Self::Current, Self::Previous];

    pub const fn role(self) -> RootSelectorRole {
        match self {
            Self::Current => RootSelectorRole::Current,
            Self::Previous => RootSelectorRole::Previous,
        }
    }

    pub const fn family(self) -> PhysicalIntegrityArtifactFamily {
        match self {
            Self::Current => PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            Self::Previous => PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Current => Self::Previous,
            Self::Previous => Self::Current,
        }
    }
}

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

pub fn selector_bytes(
    kind: SelectorKind,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> [u8; ROOT_SELECTOR_BYTES] {
    let (identity, root_generation, linked_identity, linked_generation) = match kind {
        SelectorKind::Current => (101, 11, 99, 10),
        SelectorKind::Previous => (99, 10, 97, 9),
    };
    DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(identity).unwrap(),
        kind.role(),
        root_generation,
        RootSelectorIdentity::new(linked_identity),
        Some(linked_generation),
    )
    .unwrap()
    .encode()
}

pub fn selector_scope(
    kind: SelectorKind,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> PhysicalArtifactScope {
    let range = PhysicalByteRange::new(SELECTOR_OFFSET, ROOT_SELECTOR_BYTES as u64).unwrap();
    match kind {
        SelectorKind::Current => PhysicalArtifactScope::current_root_selector(store, format, range),
        SelectorKind::Previous => {
            PhysicalArtifactScope::previous_root_selector(store, format, range)
        }
    }
}

pub fn validate_selector_rejection(
    kind: SelectorKind,
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    match kind {
        SelectorKind::Current => match validate_current_root_selector(artifact, scope) {
            (CurrentRootSelectorIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            (CurrentRootSelectorIntegrityValidation::Intact(_), _) => {
                panic!("current selector unexpectedly validated")
            }
        },
        SelectorKind::Previous => match validate_previous_root_selector(artifact, scope) {
            (PreviousRootSelectorIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            (PreviousRootSelectorIntegrityValidation::Intact(_), _) => {
                panic!("previous selector unexpectedly validated")
            }
        },
    }
}

pub fn manifest_bytes(generation: u64, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
    manifest_bytes_with_capacity(generation, format, 2)
}

pub fn manifest_bytes_with_capacity(
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
) -> Vec<u8> {
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free_space_root = FreeSpaceBlockReference::new(generation, 1, 0, 41, key, key).unwrap();
    DurablePhysicalRootManifest::builder(generation, 71, node_capacity, 43)
        .free_space_root(Some(free_space_root))
        .admit()
        .unwrap()
        .encode(format)
}

pub fn populated_manifest_bytes(
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Vec<u8> {
    let record = PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    let routing_root = ManifestBlockReference::new(generation, 1, 0, 51, record, record).unwrap();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let segment_key = SegmentPageKey::new(segment, PhysicalPageId::from_raw(1).unwrap());
    let segment_root =
        SegmentManifestBlockReference::new(generation, 1, 0, 52, segment_key, segment_key).unwrap();
    let segment_cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(generation).unwrap());
    let free_key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free_space_root =
        FreeSpaceBlockReference::new(generation, 1, 0, 41, free_key, free_key).unwrap();
    DurablePhysicalRootManifest::builder(generation, 71, 2, 43)
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

pub fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        PhysicalByteRange::new(MANIFEST_OFFSET, MANIFEST_BYTES).unwrap(),
    )
    .unwrap()
}

pub fn reseal_durable_frame(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
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
}

pub fn assert_rejected_counters(
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
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}

pub fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
