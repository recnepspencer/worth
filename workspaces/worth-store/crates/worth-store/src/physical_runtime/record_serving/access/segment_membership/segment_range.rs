use worth_store_physical_format::{
    PhysicalSegmentId, PhysicalSegmentMembershipBlock, RecordSegmentPageManifestEntry,
    SegmentManifestBlockReference,
};

use super::super::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure};
use super::SegmentMembershipReader;

impl SegmentMembershipReader<'_> {
    /// Every current membership entry of one segment, in page order.
    ///
    /// Only blocks whose key range can hold the segment are read, so the cost
    /// is the routing height plus the leaves carrying that segment's pages. A
    /// segment holds at most its admitted page capacity, which bounds the
    /// returned entries.
    pub(in crate::physical_runtime::record_serving) fn segment_entries(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        segment: PhysicalSegmentId,
        counters: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<Vec<RecordSegmentPageManifestEntry>, ManifestLookupFailure> {
        let mut entries = Vec::new();
        let Some(root) = self.root.segment_root() else {
            return Ok(entries);
        };
        let mut pending = vec![root];
        while let Some(reference) = pending.pop() {
            if !spans_segment(reference, segment) {
                continue;
            }
            match self.read_block(allocation, reference, counters)? {
                PhysicalSegmentMembershipBlock::Leaf { entries: leaf, .. } => entries.extend(
                    leaf.into_iter()
                        .filter(|entry| entry.page_cell().segment_id() == segment),
                ),
                PhysicalSegmentMembershipBlock::Branch { children, .. } => pending.extend(
                    children
                        .into_iter()
                        .rev()
                        .filter(|child| spans_segment(*child, segment)),
                ),
            }
        }
        Ok(entries)
    }
}

fn spans_segment(reference: SegmentManifestBlockReference, segment: PhysicalSegmentId) -> bool {
    reference.first().segment() <= segment && segment <= reference.last().segment()
}
