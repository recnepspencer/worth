//! Custody follows the declared chain's owners, never a measured byte total.
use super::*;
use primary_graph::{required_custody_layout_for_test, RequiredCustodyLayoutForTest};
use worth_query_host::facade::application_contribution::WorthQueryApplicationProducerBinding;

pub(super) struct ChainCustody {
    pub(super) layout: RequiredCustodyLayoutForTest,
    pub(super) root_member: usize,
    pub(super) consumer_member: usize,
}
impl ChainCustody {
    pub(super) fn new() -> Self {
        let layout = required_custody_layout_for_test();
        let root_member = layout.member_without_producer
            + crate::producer::InitialPlanarProducer::<CheckpointSchema>::IDENTITY.len();
        let consumer_member = layout.member_without_producer
            + super::super::producer::ChainProducer::<CheckpointSchema>::IDENTITY.len();
        Self {
            layout,
            root_member,
            consumer_member,
        }
    }
    fn postings(&self, addresses: &[usize]) -> usize {
        // The declared std B-tree reservation holds one spare root and one
        // node per five entries. Each address owns one refundable posting.
        let nodes = |n: usize| if n == 0 { 0 } else { 1 + n.div_ceil(5) };
        nodes(addresses.len()) * self.layout.outer_posting_node
            + addresses
                .iter()
                .map(|n| nodes(*n) * self.layout.inner_posting_node + n * self.layout.posting)
                .sum::<usize>()
    }
    pub(super) fn closed_chain(&self) -> usize {
        // C is cached; its claims keep B and A required. Each settlement
        // retains its execution context key and actual-resource slot.
        // C claims B, and B claims A, after all three callers close.
        3 * self.layout.ready
            + 2 * self.layout.source
            + self.root_member
            + self.consumer_member
            + self.postings(&[1, 1, 1])
            + 3 * self.layout.context_without_producer
            + crate::producer::InitialPlanarProducer::<CheckpointSchema>::IDENTITY.len()
            + 2 * super::super::producer::ChainProducer::<CheckpointSchema>::IDENTITY.len()
            + 3 * self.layout.settlement_slot
            + 2 * self.layout.prerequisite_slot
    }
    pub(super) fn root_refresh_before_reclaim(&self) -> usize {
        // The cached chain coexists with A's newly admitted source and member,
        // then its execution context and prepared Ready. C is the free cache.
        self.closed_chain()
            + self.layout.source
            + self.root_member
            + self.layout.context_without_producer
            + crate::producer::InitialPlanarProducer::<CheckpointSchema>::IDENTITY.len()
            + self.layout.ready
            // A's advance also prepares B's typed required successor while
            // the cached C row still owns the old chain.
            + primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<CheckpointSchema>::required_successor_custody_bytes_for_test::<super::super::producer::ChainFamily>()
            // B's selected successor source/member/context and prepared Ready
            // coexist with the predecessor chain and A's new publication.
            + self.layout.source + self.consumer_member
            + self.layout.context_without_producer
            + super::super::producer::ChainProducer::<CheckpointSchema>::IDENTITY.len()
            + self.layout.ready
            // A and B each prepare a new address before old claims retire.
            + self.postings(&[1,1,1,1,1]) - self.postings(&[1,1,1])
            + 2 * self.layout.settlement_slot + self.layout.prerequisite_slot
    }
    pub(super) fn middle_write_refresh_before_reclaim(&self) -> usize {
        // After the World writes B, the explicit ordinary B demand reads A's
        // already-current output. It needs no typed required successor or
        // overlapping A refresh. Its new address is prepared beside C's cache.
        let root = self.closed_chain()
            + self.layout.source
            + self.root_member
            + self.layout.context_without_producer
            + crate::producer::InitialPlanarProducer::<CheckpointSchema>::IDENTITY.len()
            + self.layout.ready;
        let middle = self.closed_chain()
            + self.layout.source
            + self.consumer_member
            + self.layout.context_without_producer
            + super::super::producer::ChainProducer::<CheckpointSchema>::IDENTITY.len()
            + self.layout.ready
            + self.postings(&[1, 1, 1, 1])
            - self.postings(&[1, 1, 1])
            + self.layout.settlement_slot
            + self.layout.prerequisite_slot;
        root.max(middle)
    }
    pub(super) fn middle_write_reclaim_rows(&self) -> std::ops::RangeInclusive<usize> {
        self.rows_below_peak(self.middle_write_refresh_before_reclaim())
    }
    pub(super) fn reclaim_rows(&self) -> std::ops::RangeInclusive<usize> {
        self.rows_below_peak(self.root_refresh_before_reclaim())
    }
    fn rows_below_peak(&self, peak: usize) -> std::ops::RangeInclusive<usize> {
        let row = self.layout.ready;
        // Every row-sized ceiling below the unreclaimed peak, but large enough
        // to hold the closed chain, constructs the same reclamation schedule.
        self.closed_chain().div_ceil(row)..=(peak - 1) / row
    }
}
