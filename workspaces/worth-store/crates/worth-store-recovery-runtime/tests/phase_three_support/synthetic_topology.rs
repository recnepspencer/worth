use super::*;

pub(crate) fn publish_synthetic_nonempty_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placements = [1_u64, 2].into_iter().map(placement).collect();
    let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, placements, 4).unwrap();
    let block_bytes = block.encode(format);
    let reference = block.reference(durable_artifact_checksum(&block_bytes));
    let free_key = FreeSpaceKey::arena(ExtentArenaId::new(1).unwrap(), 0);
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 4, 0x8a9b_acbd)
        .record_count(2)
        .next_block(2)
        .routing_root(Some(reference))
        .free_space_root(Some(free_space))
        .admit()
        .unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let records = root.join("families").join("records");
    let roots = records.join("roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        manifest.encode(format),
    )
    .unwrap();
    std::fs::write(
        roots.join("root-0000000000000001-block-0000000000000001.manifest"),
        block_bytes,
    )
    .unwrap();
}

pub(crate) fn publish_synthetic_branched_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let left = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 2).unwrap();
    let right =
        PhysicalRootRoutingBlock::leaf(7, 1, 2, vec![placement(2), placement(3)], 2).unwrap();
    let left_bytes = left.encode(format);
    let right_bytes = right.encode(format);
    let left_reference = left.reference(durable_artifact_checksum(&left_bytes));
    let right_reference = right.reference(durable_artifact_checksum(&right_bytes));
    let branch =
        PhysicalRootRoutingBlock::branch(7, 1, 3, 1, vec![left_reference, right_reference], 2)
            .unwrap();
    let branch_bytes = branch.encode(format);
    let branch_reference = branch.reference(durable_artifact_checksum(&branch_bytes));
    let free_key = FreeSpaceKey::arena(ExtentArenaId::new(1).unwrap(), 0);
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 2, 0x8a9b_acbd)
        .record_count(3)
        .next_block(4)
        .routing_root(Some(branch_reference))
        .free_space_root(Some(free_space))
        .admit()
        .unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let records = root.join("families").join("records");
    let roots = records.join("roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        manifest.encode(format),
    )
    .unwrap();
    for (block, bytes) in [(1, left_bytes), (2, right_bytes), (3, branch_bytes)] {
        std::fs::write(
            roots.join(format!("root-0000000000000001-block-{block:016}.manifest")),
            bytes,
        )
        .unwrap();
    }
}

fn placement(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(record, extent, 23, synthetic_range(ordinal)).unwrap(),
    )
}

fn synthetic_range(ordinal: u64) -> ExtentArenaRange {
    ExtentArenaRange::new(
        ExtentArenaId::new(1).unwrap(),
        (ordinal - 1) * (20 << 10),
        20 << 10,
    )
    .unwrap()
}
