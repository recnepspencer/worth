use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    encode_extent_chunk, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentChunkCoordinate, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
    RecordExtentGenerationCell, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::{
    validate_extent_manifest, ExtentManifestIntegrityValidation, IntegrityValidatedExtentManifest,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField, PhysicalIntegrityObservationCounters,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

pub const MANIFEST_OFFSET: u64 = 8_192;
pub const CHUNK_OFFSET: u64 = 16_384;

#[derive(Debug, Clone, Copy)]
pub struct ExtentFixture {
    pub store: StableStoreIdentity,
    pub format: PhysicalRecordFormatDeclaration,
    pub record: PersistedRecordIdentity,
    pub extent: RecordExtentGenerationCell,
    pub logical_bytes: u64,
}

impl ExtentFixture {
    pub fn new() -> Self {
        let format = format(PhysicalPageSizeClass::KiB16);
        let capacity = chunk_payload_capacity(format);
        Self {
            store: store(7),
            format,
            record: record(0x22, 7),
            extent: extent_cell(4, 5),
            logical_bytes: capacity + 5,
        }
    }

    pub fn placement(self) -> DurableExtentRecordPlacement {
        DurableExtentRecordPlacement::new(self.record, self.extent, self.logical_bytes).unwrap()
    }

    pub fn manifest(self) -> DurableExtentManifest {
        DurableExtentManifest::new(
            self.format,
            self.record,
            self.extent,
            self.logical_bytes,
            self.format.page_size().bytes(),
            2,
        )
        .unwrap()
    }

    pub fn manifest_bytes(self) -> Vec<u8> {
        self.manifest().encode(self.format)
    }

    pub fn manifest_scope(self) -> PhysicalArtifactScope {
        manifest_scope(self.store, self.format, self.placement(), 104)
    }

    pub fn chunk_coordinate(self, ordinal: u32) -> ExtentChunkCoordinate {
        let offset = match ordinal {
            1 => 0,
            2 => chunk_payload_capacity(self.format),
            _ => 0,
        };
        ExtentChunkCoordinate::new(
            self.record,
            self.extent,
            self.logical_bytes,
            offset,
            ordinal,
        )
        .unwrap()
    }

    pub fn tail_chunk_bytes(self) -> Vec<u8> {
        encode_extent_chunk(self.format, self.chunk_coordinate(2), b"tail!").unwrap()
    }

    pub fn tail_chunk_scope(self) -> PhysicalArtifactScope {
        let byte_count = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + 5;
        chunk_scope(
            self.store,
            self.format,
            self.chunk_coordinate(2),
            byte_count as u64,
        )
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

pub fn record(byte: u8, ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([byte; 16], ordinal).unwrap()
}

pub fn extent_cell(extent: u64, generation: u64) -> RecordExtentGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(generation).unwrap())
}

pub fn chunk_payload_capacity(format: PhysicalRecordFormatDeclaration) -> u64 {
    u64::from(format.page_size().bytes())
        - u64::try_from(DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES).unwrap()
}

pub fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    byte_count: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::extent_manifest(
        store,
        format,
        placement,
        PhysicalByteRange::new(MANIFEST_OFFSET, byte_count).unwrap(),
    )
}

pub fn chunk_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    coordinate: ExtentChunkCoordinate,
    byte_count: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::extent_chunk(
        store,
        format,
        coordinate,
        PhysicalByteRange::new(CHUNK_OFFSET, byte_count).unwrap(),
    )
}

pub fn validated_manifest<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedExtentManifest<'media> {
    let (validation, _) =
        validate_extent_manifest(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let ExtentManifestIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent manifest rejected")
    };
    validated
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
