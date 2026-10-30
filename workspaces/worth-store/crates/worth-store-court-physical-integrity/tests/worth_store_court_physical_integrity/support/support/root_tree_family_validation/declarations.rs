pub(crate) use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
pub(crate) use worth_store_physical_format::{
    BootstrapCatalog, CurrentPhysicalRecordPlacement, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    ManifestBlockReference, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordFormatDeclaration,
    PhysicalRootRoutingBlock, PhysicalSegmentId, PhysicalSegmentMembershipBlock,
    PhysicalTreeIdentity, RecordSegmentPageManifestEntry, RootRoutingBlockScopeIdentity,
    SegmentManifestBlockReference, SegmentMembershipBlockScopeIdentity,
};
pub(crate) use worth_store_physical_integrity::{
    validate_root_routing_block, validate_segment_membership_block,
    IntegrityValidatedRootRoutingBlock, IntegrityValidatedSegmentMembershipBlock,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField, PhysicalIntegrityObservationCounters,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    RootRoutingBlockIntegrityValidation, SegmentMembershipBlockIntegrityValidation,
    UntrustedPhysicalArtifact,
};

pub(crate) const BOOTSTRAP_OFFSET: u64 = 4_096;
pub(crate) const ROOT_BLOCK_OFFSET: u64 = 16_384;
pub(crate) const SEGMENT_BLOCK_OFFSET: u64 = 32_768;

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [byte; 16],
        )
        .unwrap(),
    )
    .published_identity()
}

pub(crate) fn format() -> PhysicalRecordFormatDeclaration {
    worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap()
}

pub(crate) fn bootstrap_bytes(store: StableStoreIdentity) -> Vec<u8> {
    worth_store_physical_format::BootstrapCatalog::new(
        store,
        format(),
        worth_store_physical_format::CurrentRootCatalogEntry::new(
            worth_store_physical_format::CurrentRootCatalogGeneration::new(11).unwrap(),
        ),
    )
    .encode()
    .to_vec()
}

pub(crate) fn bootstrap_scope(store: StableStoreIdentity, offset: u64) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::bootstrap_catalog(
        store,
        format(),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, 82).unwrap(),
    )
}

pub(crate) fn root_leaf() -> PhysicalRootRoutingBlock {
    let record = worth_store_physical_format::PersistedRecordIdentity::new([0xa1; 16], 5).unwrap();
    let extent = worth_store_physical_format::PhysicalExtentId::from_raw(19).unwrap();
    let generation = worth_store_physical_format::PhysicalGeneration::from_raw(7).unwrap();
    let cell =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(extent)
            .with_extent_generation(generation);
    let placement = worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
        record,
        cell,
        23,
        worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
            0,
            32_768,
        )
        .unwrap(),
    )
    .map(worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent)
    .unwrap();
    worth_store_physical_format::PhysicalRootRoutingBlock::leaf(71, 11, 3, vec![placement], 8)
        .unwrap()
}

pub(crate) fn root_branch(child: ManifestBlockReference) -> PhysicalRootRoutingBlock {
    worth_store_physical_format::PhysicalRootRoutingBlock::branch(71, 12, 4, 1, vec![child], 8)
        .unwrap()
}

pub(crate) fn root_scope(
    store: StableStoreIdentity,
    block: &PhysicalRootRoutingBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    root_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub(crate) fn root_scope_with_reference(
    store: StableStoreIdentity,
    reference: ManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::root_routing_block(
        store,
        format(),
        worth_store_physical_format::RootRoutingBlockScopeIdentity::new(
            worth_store_physical_format::PhysicalTreeIdentity::new(71).unwrap(),
            reference,
        ),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub(crate) fn segment_leaf() -> PhysicalSegmentMembershipBlock {
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(13).unwrap();
    let page = worth_store_physical_format::PhysicalPageId::from_raw(17).unwrap();
    let authority =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    let entry = worth_store_physical_format::RecordSegmentPageManifestEntry::new(
        authority.page_cell(segment, page).with_page_generation(
            worth_store_physical_format::PhysicalGeneration::from_raw(5).unwrap(),
        ),
        authority.segment_cell(segment).with_segment_generation(
            worth_store_physical_format::PhysicalGeneration::from_raw(6).unwrap(),
        ),
        2,
        1,
    )
    .unwrap();
    worth_store_physical_format::PhysicalSegmentMembershipBlock::leaf(73, 11, 5, vec![entry], 8)
        .unwrap()
}

pub(crate) fn segment_branch(
    child: SegmentManifestBlockReference,
) -> PhysicalSegmentMembershipBlock {
    worth_store_physical_format::PhysicalSegmentMembershipBlock::branch(
        73,
        12,
        6,
        1,
        vec![child],
        8,
    )
    .unwrap()
}

pub(crate) fn segment_scope(
    store: StableStoreIdentity,
    block: &PhysicalSegmentMembershipBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    segment_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub(crate) fn segment_scope_with_reference(
    store: StableStoreIdentity,
    reference: SegmentManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::segment_membership_block(
        store,
        format(),
        worth_store_physical_format::SegmentMembershipBlockScopeIdentity::new(
            worth_store_physical_format::PhysicalTreeIdentity::new(73).unwrap(),
            reference,
        ),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub(crate) fn reseal_durable_frame(bytes: &mut [u8]) {
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
