use std::collections::VecDeque;
use std::path::Path;

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, DurableRootSelector,
    PhysicalFreeSpaceMembershipBlock, PhysicalRootRoutingBlock, PhysicalSegmentMembershipBlock,
    RecordArtifactFile,
};

pub(super) fn required_before_successor(root: &Path) -> u64 {
    let records = root.join("families/records");
    let current = selector(&records.join("root-current.selector"));
    let previous = selector(&records.join("root-previous.selector"));
    let current_root = root_manifest(&records, current.root_generation());
    let previous_root = root_manifest(&records, previous.root_generation());

    root_leaf_entries(&records, &current_root)
        + root_leaf_entries(&records, &previous_root)
        + topology_entries(&records, &current_root)
        + topology_entries(&records, &previous_root)
}

fn selector(path: &Path) -> DurableRootSelector {
    DurableRootSelector::decode(&std::fs::read(path).expect("read selector for raw budget oracle"))
        .expect("decode selector for raw budget oracle")
}

fn root_manifest(records: &Path, generation: u64) -> DurablePhysicalRootManifest {
    let bytes = std::fs::read(
        records
            .join("roots")
            .join(RecordArtifactFile::RootManifest { generation }.file_name()),
    )
    .expect("read root for raw budget oracle");
    DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
        .expect("decode root for raw budget oracle")
        .0
}

fn root_leaf_entries(records: &Path, root: &DurablePhysicalRootManifest) -> u64 {
    let mut pending = root.routing_root().into_iter().collect::<VecDeque<_>>();
    let mut entries = 0_u64;
    while let Some(reference) = pending.pop_front() {
        let bytes = std::fs::read(
            records.join("roots").join(
                RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read routing block for raw budget oracle");
        let (block, _) = PhysicalRootRoutingBlock::decode(&bytes, root.node_capacity())
            .expect("decode routing block for raw budget oracle");
        if let Some(found) = block.entries() {
            entries += found.len() as u64;
        } else {
            pending.extend(block.children().unwrap_or_default().iter().copied());
        }
    }
    entries
}

fn topology_entries(records: &Path, root: &DurablePhysicalRootManifest) -> u64 {
    let mut entries = segment_entries(records, root);
    let header_bytes = std::fs::read(
        records.join("free-space").join(
            RecordArtifactFile::FreeSpaceManifest {
                generation: root.generation(),
            }
            .file_name(),
        ),
    )
    .expect("read free-space header for raw budget oracle");
    let (header, _) = DurableFreeSpaceManifestHeader::decode(&header_bytes, root.node_capacity())
        .expect("decode free-space header for raw budget oracle");
    let mut pending = header.root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let bytes = std::fs::read(
            records.join("free-space").join(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read free-space block for raw budget oracle");
        let (block, _) = PhysicalFreeSpaceMembershipBlock::decode(&bytes, root.node_capacity())
            .expect("decode free-space block for raw budget oracle");
        entries += block.entries().map_or_else(
            || block.children().unwrap_or_default().len(),
            |found| found.len(),
        ) as u64;
        pending.extend(block.children().unwrap_or_default().iter().copied());
    }
    entries
}

fn segment_entries(records: &Path, root: &DurablePhysicalRootManifest) -> u64 {
    let mut pending = root.segment_root().into_iter().collect::<VecDeque<_>>();
    let mut entries = 0_u64;
    while let Some(reference) = pending.pop_front() {
        let bytes = std::fs::read(
            records.join("segment-manifests").join(
                RecordArtifactFile::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .expect("read segment block for raw budget oracle");
        let (block, _) = PhysicalSegmentMembershipBlock::decode(&bytes, root.node_capacity())
            .expect("decode segment block for raw budget oracle");
        entries += block.entries().map_or_else(
            || block.children().unwrap_or_default().len(),
            |found| found.len(),
        ) as u64;
        pending.extend(block.children().unwrap_or_default().iter().copied());
    }
    entries
}
