use super::ArtifactInventory;
use worth_store_physical_format::*;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

pub(super) fn collect(inventory: &mut ArtifactInventory, root: &DurablePhysicalRootManifest) {
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).unwrap();
    if let Some(reference) = root.routing_root() {
        routing(inventory, tree, reference);
    }
    if let Some(reference) = root.segment_root() {
        segments(inventory, tree, reference);
    }
    let file = RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    };
    let scope = PhysicalArtifactScope::free_space_header(
        inventory.store,
        inventory.format,
        FreeSpaceHeaderScopeIdentity::new(
            PhysicalGeneration::from_raw(root.generation()).unwrap(),
            tree,
            root.free_space_root(),
            DurableArtifactCrc32c::new(root.free_space_checksum()),
        ),
        inventory.record_range(file),
    );
    inventory.push_record(file, "free_space_header", scope);
    if let Some(reference) = root.free_space_root() {
        free_space(inventory, tree, reference);
    }
}

fn routing(
    inventory: &mut ArtifactInventory,
    tree: PhysicalTreeIdentity,
    reference: ManifestBlockReference,
) {
    let file = RecordArtifactFile::RootRoutingBlock {
        generation: reference.generation(),
        block: reference.block(),
    };
    let scope = PhysicalArtifactScope::root_routing_block(
        inventory.store,
        inventory.format,
        RootRoutingBlockScopeIdentity::new(tree, reference),
        inventory.record_range(file),
    );
    inventory.push_record(file, "root_routing_block", scope);
    let (block, _) =
        PhysicalRootRoutingBlock::decode(&inventory.record_bytes(file), u16::MAX).unwrap();
    if let Some(children) = block.children() {
        for child in children {
            routing(inventory, tree, *child);
        }
    }
    if let Some(entries) = block.entries() {
        for entry in entries {
            if let CurrentPhysicalRecordPlacement::Extent(placement) = entry {
                super::extents::collect(inventory, *placement);
            }
        }
    }
}

fn segments(
    inventory: &mut ArtifactInventory,
    tree: PhysicalTreeIdentity,
    reference: SegmentManifestBlockReference,
) {
    let file = RecordArtifactFile::SegmentMembershipBlock {
        generation: reference.generation(),
        block: reference.block(),
    };
    let scope = PhysicalArtifactScope::segment_membership_block(
        inventory.store,
        inventory.format,
        SegmentMembershipBlockScopeIdentity::new(tree, reference),
        inventory.record_range(file),
    );
    inventory.push_record(file, "segment_membership_block", scope);
    let (block, _) =
        PhysicalSegmentMembershipBlock::decode(&inventory.record_bytes(file), u16::MAX).unwrap();
    if let Some(children) = block.children() {
        for child in children {
            segments(inventory, tree, *child);
        }
    }
    if let Some(entries) = block.entries() {
        for entry in entries {
            let file = RecordArtifactFile::Segment {
                segment: entry.page_cell().segment_id().get(),
                generation: entry.data_generation(),
            };
            let page_bytes = inventory.format.page_size().bytes() as u64;
            let range = PhysicalByteRange::new(entry.frame_index() as u64 * page_bytes, page_bytes)
                .unwrap();
            inventory.push_record(
                file,
                "inline_page",
                PhysicalArtifactScope::inline_page(
                    inventory.store,
                    inventory.format,
                    entry.page_cell(),
                    range,
                ),
            );
        }
    }
}

fn free_space(
    inventory: &mut ArtifactInventory,
    tree: PhysicalTreeIdentity,
    reference: FreeSpaceBlockReference,
) {
    let file = RecordArtifactFile::FreeSpaceMembershipBlock {
        generation: reference.generation(),
        block: reference.block(),
    };
    let scope = PhysicalArtifactScope::free_space_membership_block(
        inventory.store,
        inventory.format,
        FreeSpaceMembershipBlockScopeIdentity::new(tree, reference),
        inventory.record_range(file),
    );
    inventory.push_record(file, "free_space_membership_block", scope);
    let (block, _) =
        PhysicalFreeSpaceMembershipBlock::decode(&inventory.record_bytes(file), u16::MAX).unwrap();
    if let Some(children) = block.children() {
        for child in children {
            free_space(inventory, tree, *child);
        }
    }
}
