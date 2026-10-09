//! An observed root and inventory for the walk's unit tests: headers and
//! segment pages only.

use std::collections::BTreeMap;

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, FreeSpaceBlockReference,
    FreeSpaceKey, ManifestBlockReference, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalSegmentId, RecordArtifactFile,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};

use crate::progression::{RecoverySelectedSegmentPage, RecoverySelectedSourceInventory};

/// A root of this node capacity over a leaf of `records` records.
pub(super) fn root(node_capacity: u16, records: u64) -> DurablePhysicalRootManifest {
    root_at(1, node_capacity, records)
}

/// The same root at another generation.
pub(super) fn root_at(
    generation: u64,
    node_capacity: u16,
    records: u64,
) -> DurablePhysicalRootManifest {
    let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal);
    let leaf =
        ManifestBlockReference::new(1, 1, 0, 99, record(1).unwrap(), record(records).unwrap());
    DurablePhysicalRootManifest::builder(generation, 7, node_capacity, 19)
        .record_count(records)
        .next_block(2)
        .routing_root(leaf)
        .admit()
        .unwrap()
}

/// `pages` pages of one segment under a free-space header of this node
/// capacity that counts `free_entries` entries under one leaf.
pub(super) fn inventory(
    free_node_capacity: u16,
    pages: u64,
    free_entries: u64,
) -> RecoverySelectedSourceInventory {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let segment_pages = (1..=pages)
        .map(|page| {
            let entry = RecordSegmentPageManifestEntry::new(
                authority
                    .page_cell(segment, PhysicalPageId::from_raw(page).unwrap())
                    .with_page_generation(generation),
                authority
                    .segment_cell(segment)
                    .with_segment_generation(generation),
                1,
                0,
            )
            .unwrap();
            let observed = RecoverySelectedSegmentPage {
                entry,
                routing_identity: [0; 32],
                membership_artifact: RecordArtifactFile::RootManifest { generation: 1 },
            };
            ((1, page), observed)
        })
        .collect();
    let key =
        FreeSpaceKey::from(RecordFreeSpaceManifestEntry::inline_frontier(1, 1, 2, 8).unwrap());
    let leaf =
        (free_entries != 0).then(|| FreeSpaceBlockReference::new(1, 1, 0, 9, key, key).unwrap());
    RecoverySelectedSourceInventory {
        free_space: DurableFreeSpaceManifestHeader::new(
            1,
            7,
            free_node_capacity,
            4,
            free_entries,
            1,
            1,
            1,
            1,
            32768,
            4096,
            2,
            leaf,
        )
        .unwrap(),
        segment_pages,
        segment_topology: BTreeMap::new(),
        free_entries: Box::new([]),
        free_topology: BTreeMap::new(),
        source_artifacts: Box::new([]),
    }
}
