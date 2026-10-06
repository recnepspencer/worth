use std::mem::size_of;

use worth_foundational::PartitionIdentity;

use super::{
    cut_tree::{ordered_pair, CutTree},
    Bisection, WeightedEdge, WeightedItem,
};
use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::KernelContext,
    partition::{
        checked_scope::checked_edit, PartitionItemId, PartitionUpdateDenial, PartitionWork,
    },
};

// Each item can appear in at most 64 cut-node member sets because cut paths
// are u64. Sixteen item slots per level cover a singleton BTree node's slack,
// while the remainder covers item/route maps and at most two cut nodes per
// item. Edge slots cover the edge map and both incident indexes.
const ITEM_RETAINED_BYTES: usize = 64 * 16 * size_of::<PartitionItemId>()
    + 8 * (size_of::<WeightedItem>()
        + 2 * size_of::<CutTree>()
        + size_of::<(PartitionItemId, PartitionIdentity)>());
const EDGE_RETAINED_BYTES: usize =
    16 * (size_of::<WeightedEdge>() + 2 * size_of::<(PartitionItemId, PartitionItemId)>());

impl Bisection {
    pub fn upsert_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: WeightedItem,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let count = self
                .items
                .len()
                .checked_add(usize::from(!self.items.contains_key(&item.item)))
                .ok_or(PartitionUpdateDenial::Admission(
                    LeaseDenial::ChargedBytesOverflow,
                ))?;
            let bound = self.retained_bytes(count, self.edges.len())?;
            self.retained_charge.admit(lease, bound)?;
            self.upsert_item_structural_checked(lease, child, item)
        })
    }

    pub fn remove_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let current = self.retained_bytes(self.items.len(), self.edges.len())?;
            self.retained_charge.admit(lease, current)?;
            let work = self.remove_item_structural_checked(lease, child, item)?;
            let next = self.retained_bytes(self.items.len(), self.edges.len())?;
            self.retained_charge
                .admit(lease, next)
                .expect("same-lineage shrink");
            Ok(work)
        })
    }

    pub fn upsert_edge_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        edge: WeightedEdge,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let key = ordered_pair(edge.a, edge.b);
            let count = self
                .edges
                .len()
                .checked_add(usize::from(!self.edges.contains_key(&key)))
                .ok_or(PartitionUpdateDenial::Admission(
                    LeaseDenial::ChargedBytesOverflow,
                ))?;
            let bound = self.retained_bytes(self.items.len(), count)?;
            self.retained_charge.admit(lease, bound)?;
            self.upsert_edge_structural_checked(lease, child, edge)
        })
    }

    pub fn remove_edge_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let current = self.retained_bytes(self.items.len(), self.edges.len())?;
            self.retained_charge.admit(lease, current)?;
            let work = self.remove_edge_structural_checked(child, a, b)?;
            let next = self.retained_bytes(self.items.len(), self.edges.len())?;
            self.retained_charge
                .admit(lease, next)
                .expect("same-lineage shrink");
            Ok(work)
        })
    }

    fn retained_bytes(&self, items: usize, edges: usize) -> Result<u64, PartitionUpdateDenial> {
        items
            .checked_mul(ITEM_RETAINED_BYTES)
            .and_then(|bytes| bytes.checked_add(edges.checked_mul(EDGE_RETAINED_BYTES)?))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(PartitionUpdateDenial::Admission(
                LeaseDenial::ChargedBytesOverflow,
            ))
    }
}
