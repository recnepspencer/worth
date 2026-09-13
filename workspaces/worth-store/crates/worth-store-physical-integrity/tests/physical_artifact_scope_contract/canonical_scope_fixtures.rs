use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableExtentRecordPlacement, ExtentChunkCoordinate,
    FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
    FreeSpaceMembershipBlockScopeIdentity, ManifestBlockReference, PageGenerationCell,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordFormatDeclaration, PhysicalSegmentId, PhysicalTreeIdentity,
    RecordAllocationClass, RootRoutingBlockScopeIdentity, SegmentManifestBlockReference,
    SegmentMembershipBlockScopeIdentity, SegmentPageKey,
};
use worth_store_physical_integrity::PhysicalByteRange;

pub(super) fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(super) fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

pub(super) fn range(offset: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(offset, 64).unwrap()
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

pub(super) fn page() -> PageGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            PhysicalSegmentId::from_raw(2).unwrap(),
            PhysicalPageId::from_raw(3).unwrap(),
        )
        .with_page_generation(PhysicalGeneration::from_raw(4).unwrap())
}

pub(super) fn extent_placement() -> DurableExtentRecordPlacement {
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(5).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(6).unwrap());
    DurableExtentRecordPlacement::new(record(7), extent, 1024).unwrap()
}

pub(super) fn extent_chunk() -> ExtentChunkCoordinate {
    ExtentChunkCoordinate::new(record(7), extent_placement().extent_cell(), 1024, 0, 1).unwrap()
}

fn root_block() -> ManifestBlockReference {
    ManifestBlockReference::new(11, 12, 0, 13, record(1), record(2)).unwrap()
}

fn segment_block() -> SegmentManifestBlockReference {
    SegmentManifestBlockReference::new(
        14,
        15,
        0,
        16,
        SegmentPageKey::new(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        ),
        SegmentPageKey::new(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(2).unwrap(),
        ),
    )
    .unwrap()
}

fn free_space_block() -> FreeSpaceBlockReference {
    FreeSpaceBlockReference::new(
        17,
        18,
        0,
        19,
        FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap(),
        FreeSpaceKey::new(RecordAllocationClass::Extent, 2).unwrap(),
    )
    .unwrap()
}

fn tree() -> PhysicalTreeIdentity {
    PhysicalTreeIdentity::new(10).unwrap()
}

pub(super) fn root_routing_identity() -> RootRoutingBlockScopeIdentity {
    RootRoutingBlockScopeIdentity::new(tree(), root_block())
}

pub(super) fn segment_membership_identity() -> SegmentMembershipBlockScopeIdentity {
    SegmentMembershipBlockScopeIdentity::new(tree(), segment_block())
}

pub(super) fn free_space_header_identity() -> FreeSpaceHeaderScopeIdentity {
    FreeSpaceHeaderScopeIdentity::new(
        PhysicalGeneration::from_raw(22).unwrap(),
        tree(),
        Some(free_space_block()),
        DurableArtifactCrc32c::new(23),
    )
}

pub(super) fn free_space_membership_identity() -> FreeSpaceMembershipBlockScopeIdentity {
    FreeSpaceMembershipBlockScopeIdentity::new(tree(), free_space_block())
}
