//! The canonical forms the inventory checks and keys by: free runs in
//! canonical order, and the routing identity of each segment page.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, FreeSpaceKey, PhysicalRecordFormatDeclaration,
    RecordFreeSpaceManifestEntry, SegmentManifestBlockReference,
};

/// Arena ownership is keyed by offset as well as arena id. Several disjoint
/// free runs in one arena are legal; duplicates, overlaps and uncoalesced
/// touching runs are not.
pub(super) fn canonical_free_entries(entries: &mut [RecordFreeSpaceManifestEntry]) -> bool {
    entries.sort_unstable_by_key(|entry| FreeSpaceKey::from(*entry));
    entries.windows(2).all(|pair| {
        let left = pair[0];
        let right = pair[1];
        FreeSpaceKey::from(left) < FreeSpaceKey::from(right)
            && match (left.arena_free_range(), right.arena_free_range()) {
                (Some(left), Some(right)) if left.arena() == right.arena() => {
                    left.end() < right.offset()
                }
                _ => true,
            }
    })
}

pub(super) fn routing_identity(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    reference: SegmentManifestBlockReference,
    entry: worth_store_physical_format::RecordSegmentPageManifestEntry,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.segment-page-routing.v1");
    digest.update(root.encode(format));
    digest.update(reference.generation().to_le_bytes());
    digest.update(reference.block().to_le_bytes());
    digest.update(reference.level().to_le_bytes());
    digest.update(reference.checksum().to_le_bytes());
    digest.update(reference.first().segment().get().to_le_bytes());
    digest.update(reference.first().page().get().to_le_bytes());
    digest.update(reference.last().segment().get().to_le_bytes());
    digest.update(reference.last().page().get().to_le_bytes());
    digest.update(entry.page_cell().segment_id().get().to_le_bytes());
    digest.update(entry.page().get().to_le_bytes());
    digest.update(entry.page_generation().to_le_bytes());
    digest.update(entry.data_generation().to_le_bytes());
    digest.update(entry.data_page_count().to_le_bytes());
    digest.update(entry.frame_index().to_le_bytes());
    digest.finalize().into()
}
