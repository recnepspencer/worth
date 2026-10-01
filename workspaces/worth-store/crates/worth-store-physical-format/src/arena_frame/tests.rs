use crate::*;

#[test]
fn arena_geometry_carries_alignment_without_rounding_payload_into_authority() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let layout = ExtentArenaFrameLayout::new(format, 4096).unwrap();
    assert_eq!(layout.manifest_stride(), 4096);
    assert_eq!(layout.chunk_stride(), 16384);
    assert_eq!(layout.allocated_bytes(2), Some(36864));
    assert_eq!(layout.chunk_offset(2), Some(20480));
    assert_eq!(layout.chunk_offset(0), None);
    assert!(ExtentArenaFrameLayout::new(format, 3).is_none());
    assert!(ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), u64::MAX, 1).is_none());
    assert!(!layout.admits(range(1, 1, 36864), 2));
    assert!(!layout.admits(range(1, 0, 36865), 2));
}

#[test]
fn published_free_runs_are_sorted_disjoint_coalesced_and_keep_zero_offset() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let entry = |offset, length| {
        RecordFreeSpaceManifestEntry::arena_range(range(1, offset, length), 2).unwrap()
    };
    let block =
        PhysicalFreeSpaceMembershipBlock::leaf(1, 2, 1, vec![entry(0, 4096), entry(8192, 4096)], 4)
            .unwrap();
    let encoded = block.encode(format);
    assert_eq!(
        PhysicalFreeSpaceMembershipBlock::decode(&encoded, 4)
            .unwrap()
            .0,
        block
    );
    assert!(PhysicalFreeSpaceMembershipBlock::leaf(
        1,
        2,
        1,
        vec![entry(0, 8192), entry(4096, 4096)],
        4
    )
    .is_none());
    assert!(PhysicalFreeSpaceMembershipBlock::leaf(
        1,
        2,
        1,
        vec![entry(0, 4096), entry(4096, 4096)],
        4
    )
    .is_none());
    assert!(PhysicalFreeSpaceMembershipBlock::leaf(
        1,
        2,
        1,
        vec![entry(8192, 4096), entry(0, 4096)],
        4
    )
    .is_none());
}

#[test]
fn root_routes_preserve_arena_range_and_old_store_version_is_denied() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
    let route =
        DurableExtentRecordPlacement::legacy_unknown(record, extent, 3, range(9, 4096, 20480))
            .unwrap();
    let block = PhysicalRootRoutingBlock::leaf(
        1,
        2,
        1,
        vec![CurrentPhysicalRecordPlacement::Extent(route)],
        2,
    )
    .unwrap();
    let encoded = block.encode(format);
    assert_eq!(&encoded[120..128], &9_u64.to_le_bytes());
    assert_eq!(&encoded[144..152], &4096_u64.to_le_bytes());
    assert_eq!(&encoded[152..160], &20480_u64.to_le_bytes());
    assert_eq!(
        PhysicalRootRoutingBlock::decode(&encoded, 2).unwrap().0,
        block
    );
    let mut prior = encoded;
    prior[10..12].copy_from_slice(&1_u16.to_le_bytes());
    assert!(matches!(
        PhysicalRootRoutingBlock::decode(&prior, 2),
        Err(RootRoutingBlockDenial::Frame(
            DurableFrameDenial::UnsupportedFormat(PhysicalRecordFormatDenial::UnsupportedVersion(
                1
            ))
        ))
    ));
}

fn range(arena: u64, offset: u64, length: u64) -> ExtentArenaRange {
    ExtentArenaRange::new(ExtentArenaId::new(arena).unwrap(), offset, length).unwrap()
}

#[test]
fn exhausted_arena_header_preserves_geometry_without_free_runs() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let header =
        DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 1, 1, 2, 2, 65536, 4096, 1, None)
            .unwrap();
    let encoded = header.encode(format);
    assert_eq!(encoded.len(), 216);
    assert_eq!(&encoded[200..208], &65536_u64.to_le_bytes());
    assert_eq!(&encoded[208..216], &4096_u64.to_le_bytes());
    let decoded = DurableFreeSpaceManifestHeader::decode(&encoded, 2)
        .unwrap()
        .0;
    assert_eq!(decoded.next_arena(), 2);
    assert_eq!(decoded.arena_capacity(), 65536);
    assert_eq!(decoded.arena_alignment(), 4096);
    assert!(decoded.root().is_none());
    let root = DurablePhysicalRootManifest::builder(1, 1, 2, durable_artifact_checksum(&encoded))
        .admit()
        .unwrap();
    assert!(root.free_space_root().is_none());
    assert_eq!(
        DurablePhysicalRootManifest::decode(&root.encode(format), 2)
            .unwrap()
            .0,
        root
    );
    assert!(
        DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 1, 1, 2, 2, 65535, 4096, 1, None)
            .is_none()
    );
    assert!(
        DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 1, 1, 2, 2, 65536, 3, 1, None).is_none()
    );
}

#[test]
fn fully_occupied_extent_arena_root_has_no_free_tree() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = PersistedRecordIdentity::new([3; 16], 1).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let placement =
        DurableExtentRecordPlacement::legacy_unknown(record, extent, 3, range(1, 0, 20480))
            .unwrap();
    let routing = PhysicalRootRoutingBlock::leaf(
        1,
        1,
        1,
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        2,
    )
    .unwrap();
    let reference = routing.reference(durable_artifact_checksum(&routing.encode(format)));
    let free = DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 1, 1, 2, 2, 20480, 4096, 1, None)
        .unwrap();
    let root = DurablePhysicalRootManifest::builder(
        1,
        1,
        2,
        durable_artifact_checksum(&free.encode(format)),
    )
    .record_count(1)
    .next_block(2)
    .routing_root(Some(reference))
    .admit()
    .unwrap();
    assert!(root.free_space_root().is_none());
    assert_eq!(
        DurablePhysicalRootManifest::decode(&root.encode(format), 2)
            .unwrap()
            .0,
        root
    );
}
