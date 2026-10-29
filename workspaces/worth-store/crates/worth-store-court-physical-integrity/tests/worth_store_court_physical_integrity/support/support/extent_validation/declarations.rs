pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RecordExtentGenerationCell, worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};
pub(crate) use {worth_store_physical_integrity::validate_extent_manifest, worth_store_physical_integrity::ExtentManifestIntegrityValidation, worth_store_physical_integrity::IntegrityValidatedExtentManifest, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) const MANIFEST_OFFSET: u64 = 65_536;
pub(crate) const CHUNK_OFFSET: u64 = 98_304;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExtentFixture {
    pub(crate) store: StableStoreIdentity,
    pub(crate) format: PhysicalRecordFormatDeclaration,
    pub(crate) record: PersistedRecordIdentity,
    pub(crate) extent: RecordExtentGenerationCell,
    pub(crate) logical_bytes: u64,
}

impl ExtentFixture {
    pub(crate) fn new() -> Self {
        let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
        let capacity = chunk_payload_capacity(format);
        Self {
            store: store(7),
            format,
            record: record(0x22, 7),
            extent: extent_cell(4, 5),
            logical_bytes: capacity + 5,
        }
    }

    pub(crate) fn placement(self) -> DurableExtentRecordPlacement {
        worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
            self.record,
            self.extent,
            self.logical_bytes,
            self.arena_range(),
        )
        .unwrap()
    }

    pub(crate) fn arena_range(self) -> ExtentArenaRange {
        let layout =
            worth_store_physical_format::ExtentArenaFrameLayout::new(self.format, u64::from(self.format.page_size().bytes()))
                .unwrap();
        let count = self.manifest().chunk_count();
        worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
            MANIFEST_OFFSET,
            layout.allocated_bytes(count).unwrap(),
        )
        .unwrap()
    }

    pub(crate) fn manifest(self) -> DurableExtentManifest {
        worth_store_physical_format::DurableExtentManifest::new(
            self.format,
            self.record,
            self.extent,
            self.logical_bytes,
            self.format.page_size().bytes(),
            2,
            u64::from(self.format.page_size().bytes()),
        )
        .unwrap()
    }

    pub(crate) fn manifest_bytes(self) -> Vec<u8> {
        self.manifest().encode(self.format)
    }

    pub(crate) fn manifest_scope(self) -> PhysicalArtifactScope {
        manifest_scope(self.store, self.format, self.placement(), 104)
    }

    pub(crate) fn chunk_coordinate(self, ordinal: u32) -> ExtentChunkCoordinate {
        let offset = match ordinal {
            1 => 0,
            2 => chunk_payload_capacity(self.format),
            _ => 0,
        };
        worth_store_physical_format::ExtentChunkCoordinate::new(
            self.record,
            self.extent,
            self.logical_bytes,
            offset,
            ordinal,
        )
        .unwrap()
    }

    pub(crate) fn tail_chunk_bytes(self) -> Vec<u8> {
        encode_extent_chunk(self.format, self.chunk_coordinate(2), b"tail!").unwrap()
    }

    pub(crate) fn tail_chunk_scope(self) -> PhysicalArtifactScope {
        let byte_count = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + 5;
        chunk_scope(
            self.store,
            self.format,
            self.chunk_coordinate(2),
            byte_count as u64,
        )
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

pub(crate) fn record(byte: u8, ordinal: u64) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([byte; 16], ordinal).unwrap()
}

pub(crate) fn extent_cell(extent: u64, generation: u64) -> RecordExtentGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(worth_store_physical_format::PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(worth_store_physical_format::PhysicalGeneration::from_raw(generation).unwrap())
}

pub(crate) fn chunk_payload_capacity(format: PhysicalRecordFormatDeclaration) -> u64 {
    u64::from(format.page_size().bytes())
        - u64::try_from(DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES).unwrap()
}

pub(crate) fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    byte_count: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(
        store,
        format,
        placement,
        worth_store_physical_integrity::PhysicalByteRange::new(MANIFEST_OFFSET, byte_count).unwrap(),
    )
}

pub(crate) fn chunk_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    coordinate: ExtentChunkCoordinate,
    byte_count: u64,
) -> PhysicalArtifactScope {
    let layout =
        worth_store_physical_format::ExtentArenaFrameLayout::new(format, u64::from(format.page_size().bytes())).unwrap();
    let capacity = chunk_payload_capacity(format);
    let count = u32::try_from(coordinate.logical_bytes().div_ceil(capacity)).unwrap();
    let arena_range = worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
        MANIFEST_OFFSET,
        layout.allocated_bytes(count).unwrap(),
    )
    .unwrap();
    worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
        store,
        format,
        coordinate,
        worth_store_physical_integrity::PhysicalByteRange::new(
            MANIFEST_OFFSET + layout.chunk_offset(coordinate.ordinal()).unwrap(),
            byte_count,
        )
        .unwrap(),
        arena_range,
    )
}

pub(crate) fn literal_arena_range(format: PhysicalRecordFormatDeclaration) -> ExtentArenaRange {
    let layout =
        worth_store_physical_format::ExtentArenaFrameLayout::new(format, u64::from(format.page_size().bytes())).unwrap();
    worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
        MANIFEST_OFFSET,
        layout.allocated_bytes(1).unwrap(),
    )
    .unwrap()
}

pub(crate) fn validated_manifest<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedExtentManifest<'media> {
    let (validation, _) =
        validate_extent_manifest(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let worth_store_physical_integrity::ExtentManifestIntegrityValidation::Intact(validated) = validation else {
        panic!("clean extent manifest rejected")
    };
    validated
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