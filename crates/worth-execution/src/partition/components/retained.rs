use super::{
    checked::{DEGREE_SCRATCH_BYTES, MEMBER_SCRATCH_BYTES},
    ComponentPartitioner,
};
use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::KernelContext,
    partition::{
        checked_scope::checked_edit, PartitionItemId, PartitionUpdateDenial, PartitionWork,
        SourceFactId,
    },
};

impl ComponentPartitioner {
    /// Admitted runtime item edit. The owned live-graph charge persists across
    /// calls and moves atomically to a later request lease when needed.
    pub fn upsert_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
        fact: SourceFactId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let count = self
                .facts
                .len()
                .checked_add(usize::from(!self.facts.contains_key(&item)))
                .ok_or(PartitionUpdateDenial::Admission(
                    LeaseDenial::ChargedBytesOverflow,
                ))?;
            let bound = self.retained_bytes(count, self.edge_count)?;
            self.retained_charge.admit(lease, bound)?;
            self.upsert_item_structural_checked(lease, child, item, fact)
        })
    }

    /// Admitted runtime merge; the new edge's retained capacity is secured
    /// before candidate traversal and any route rewrite.
    pub fn add_edge_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let add = usize::from(
                self.adjacent
                    .get(&a)
                    .is_some_and(|neighbors| !neighbors.contains(&b)),
            );
            let edges =
                self.edge_count
                    .checked_add(add)
                    .ok_or(PartitionUpdateDenial::Admission(
                        LeaseDenial::ChargedBytesOverflow,
                    ))?;
            let bound = self.retained_bytes(self.facts.len(), edges)?;
            self.retained_charge.admit(lease, bound)?;
            self.add_edge_structural_checked(lease, child, a, b)
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
            let current = self.retained_bytes(self.facts.len(), self.edge_count)?;
            self.retained_charge.admit(lease, current)?;
            let work = self.remove_edge_structural_checked(lease, child, a, b)?;
            let next = self.retained_bytes(self.facts.len(), self.edge_count)?;
            self.retained_charge
                .admit(lease, next)
                .expect("same-lineage shrink");
            Ok(work)
        })
    }

    pub fn remove_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            let current = self.retained_bytes(self.facts.len(), self.edge_count)?;
            self.retained_charge.admit(lease, current)?;
            let work = self.remove_item_structural_checked(lease, child, item)?;
            let next = self.retained_bytes(self.facts.len(), self.edge_count)?;
            self.retained_charge
                .admit(lease, next)
                .expect("same-lineage shrink");
            Ok(work)
        })
    }

    fn retained_bytes(&self, members: usize, edges: usize) -> Result<u64, PartitionUpdateDenial> {
        members
            .checked_mul(MEMBER_SCRATCH_BYTES)
            .and_then(|bytes| bytes.checked_add(edges.checked_mul(2 * DEGREE_SCRATCH_BYTES)?))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(PartitionUpdateDenial::Admission(
                LeaseDenial::ChargedBytesOverflow,
            ))
    }
}
