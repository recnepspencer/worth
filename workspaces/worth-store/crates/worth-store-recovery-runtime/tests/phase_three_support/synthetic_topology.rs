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
    write_genesis(root, store, format, &manifest, vec![block_bytes]);
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
    write_genesis(
        root,
        store,
        format,
        &manifest,
        vec![left_bytes, right_bytes, branch_bytes],
    );
}

/// One leaf of one placement under `branches` branches of one child each,
/// the root last, under a manifest whose record count asks for that height.
/// Every block is sound on its own; the leaves hold fewer entries than the
/// manifest counts, which only a whole walk can see.
pub(crate) fn publish_synthetic_chained_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    branches: u16,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let leaf = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 2).unwrap();
    let mut blocks = vec![leaf.encode(format)];
    let mut reference = leaf.reference(durable_artifact_checksum(&blocks[0]));
    for level in 1..=branches {
        let block = u64::from(level) + 1;
        let branch = PhysicalRootRoutingBlock::branch(7, 1, block, level, vec![reference], 2)
            .expect("a branch of one child is a sound routing block");
        let encoded = branch.encode(format);
        reference = branch.reference(durable_artifact_checksum(&encoded));
        blocks.push(encoded);
    }
    let free_key = FreeSpaceKey::arena(ExtentArenaId::new(1).unwrap(), 0);
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    // The manifest fixes the root's level as the least height that covers its
    // record count; one record more than a full tree one level shorter holds
    // asks for exactly this chain's height.
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 2, 0x8a9b_acbd)
        .record_count((1_u64 << branches) + 1)
        .next_block(blocks.len() as u64 + 1)
        .routing_root(Some(reference))
        .free_space_root(Some(free_space))
        .admit()
        .unwrap();
    write_genesis(root, store, format, &manifest, blocks);
}

/// Leaves holding `leaves` entries each, in order, under branches of two
/// children until one root, under a manifest counting `record_count`.
pub(crate) fn publish_synthetic_paired_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    record_count: u64,
    leaves: &[u64],
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let mut blocks = Vec::new();
    let mut level = Vec::new();
    let mut ordinal = 0;
    for &count in leaves {
        let placements = (ordinal + 1..=ordinal + count).map(placement).collect();
        ordinal += count;
        let leaf = PhysicalRootRoutingBlock::leaf(7, 1, blocks.len() as u64 + 1, placements, 2)
            .expect("a leaf of one or two ordered entries");
        let encoded = leaf.encode(format);
        level.push(leaf.reference(durable_artifact_checksum(&encoded)));
        blocks.push(encoded);
    }
    let mut height = 0;
    while level.len() > 1 {
        height += 1;
        let children = std::mem::take(&mut level);
        for pair in children.chunks(2) {
            let block = blocks.len() as u64 + 1;
            let branch = PhysicalRootRoutingBlock::branch(7, 1, block, height, pair.to_vec(), 2)
                .expect("a branch of one or two ordered children");
            let encoded = branch.encode(format);
            level.push(branch.reference(durable_artifact_checksum(&encoded)));
            blocks.push(encoded);
        }
    }
    let free_key = FreeSpaceKey::arena(ExtentArenaId::new(1).unwrap(), 0);
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 2, 0x8a9b_acbd)
        .record_count(record_count)
        .next_block(blocks.len() as u64 + 1)
        .routing_root(Some(level[0]))
        .free_space_root(Some(free_space))
        .admit()
        .expect("the record count asks for the paired tree's height");
    write_genesis(root, store, format, &manifest, blocks);
}

/// Writes the current selector, its root manifest, and its routing blocks
/// numbered from one in order.
fn write_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    manifest: &DurablePhysicalRootManifest,
    blocks: Vec<Vec<u8>>,
) {
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
    for (index, bytes) in blocks.into_iter().enumerate() {
        let block = index + 1;
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
        DurableExtentRecordPlacement::legacy_unknown(record, extent, 23, synthetic_range(ordinal))
            .unwrap(),
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
