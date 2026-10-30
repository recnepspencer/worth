pub(crate) use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
pub(crate) use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    ExtentChunkCoordinate, FreeSpaceBlockReference, FreeSpaceHeaderScopeIdentity, FreeSpaceKey,
    FreeSpaceMembershipBlockScopeIdentity, ManifestBlockReference, PageGenerationCell,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordFormatDeclaration, PhysicalSegmentId, PhysicalTreeIdentity,
    RootRoutingBlockScopeIdentity, SegmentManifestBlockReference,
    SegmentMembershipBlockScopeIdentity, SegmentPageKey,
};
pub(crate) use worth_store_physical_integrity::PhysicalByteRange;

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

pub(crate) fn range(offset: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset, 64).unwrap()
}

pub(crate) fn record(ordinal: u64) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

pub(crate) fn page() -> PageGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            worth_store_physical_format::PhysicalSegmentId::from_raw(2).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(3).unwrap(),
        )
        .with_page_generation(worth_store_physical_format::PhysicalGeneration::from_raw(4).unwrap())
}

pub(crate) fn extent_placement() -> DurableExtentRecordPlacement {
    let extent =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(worth_store_physical_format::PhysicalExtentId::from_raw(5).unwrap())
            .with_extent_generation(
                worth_store_physical_format::PhysicalGeneration::from_raw(6).unwrap(),
            );
    worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
        record(7),
        extent,
        1024,
        arena_range(),
    )
    .unwrap()
}

pub(crate) fn arena_range() -> ExtentArenaRange {
    worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
        32_768,
        32_768,
    )
    .unwrap()
}

pub(crate) fn extent_chunk() -> ExtentChunkCoordinate {
    worth_store_physical_format::ExtentChunkCoordinate::new(
        record(7),
        extent_placement().extent_cell(),
        1024,
        0,
        1,
    )
    .unwrap()
}

pub(crate) fn root_block() -> ManifestBlockReference {
    worth_store_physical_format::ManifestBlockReference::new(11, 12, 0, 13, record(1), record(2))
        .unwrap()
}

pub(crate) fn segment_block() -> SegmentManifestBlockReference {
    worth_store_physical_format::SegmentManifestBlockReference::new(
        14,
        15,
        0,
        16,
        worth_store_physical_format::SegmentPageKey::new(
            worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap(),
        ),
        worth_store_physical_format::SegmentPageKey::new(
            worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(2).unwrap(),
        ),
    )
    .unwrap()
}

pub(crate) fn free_space_block() -> FreeSpaceBlockReference {
    worth_store_physical_format::FreeSpaceBlockReference::new(
        17,
        18,
        0,
        19,
        worth_store_physical_format::FreeSpaceKey::inline(1).unwrap(),
        worth_store_physical_format::FreeSpaceKey::arena(
            worth_store_physical_format::ExtentArenaId::new(2).unwrap(),
            0,
        ),
    )
    .unwrap()
}

pub(crate) fn tree() -> PhysicalTreeIdentity {
    worth_store_physical_format::PhysicalTreeIdentity::new(10).unwrap()
}

pub(crate) fn root_routing_identity() -> RootRoutingBlockScopeIdentity {
    worth_store_physical_format::RootRoutingBlockScopeIdentity::new(tree(), root_block())
}

pub(crate) fn segment_membership_identity() -> SegmentMembershipBlockScopeIdentity {
    worth_store_physical_format::SegmentMembershipBlockScopeIdentity::new(tree(), segment_block())
}

pub(crate) fn free_space_header_identity() -> FreeSpaceHeaderScopeIdentity {
    worth_store_physical_format::FreeSpaceHeaderScopeIdentity::new(
        worth_store_physical_format::PhysicalGeneration::from_raw(22).unwrap(),
        tree(),
        Some(free_space_block()),
        worth_store_physical_format::DurableArtifactCrc32c::new(23),
    )
}

pub(crate) fn free_space_membership_identity() -> FreeSpaceMembershipBlockScopeIdentity {
    worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity::new(
        tree(),
        free_space_block(),
    )
}
