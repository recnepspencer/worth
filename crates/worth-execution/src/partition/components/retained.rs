use super::{
    checked::{DEGREE_SCRATCH_BYTES, MEMBER_SCRATCH_BYTES},
    request::ComponentFunding,
    ComponentPartitioner,
};
use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::KernelContext,
    partition::{
        checked_scope::checked_edit, PartitionItemId, PartitionUpdateDenial, PartitionWork,
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
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            self.upsert_item_admitted(&ComponentFunding::leased(lease), child, item)
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
            self.add_edge_admitted(&ComponentFunding::leased(lease), child, a, b)
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
            self.remove_edge_admitted(&ComponentFunding::leased(lease), child, a, b)
        })
    }

    pub fn remove_item_checked(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        checked_edit(lease, context, |child| {
            self.remove_item_admitted(&ComponentFunding::leased(lease), child, item)
        })
    }

    pub(super) fn upsert_item_admitted(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let count = self
            .routes
            .len()
            .checked_add(usize::from(!self.routes.contains_key(&item)))
            .ok_or_else(|| {
                PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
            })?;
        let bound = self.retained_bytes(count, self.edge_count)?;
        funding.retained(&mut self.retained_charge, bound)?;
        self.upsert_item_structural_checked(funding, context, item)
    }

    pub(super) fn add_edge_admitted(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let add = usize::from(
            self.adjacent
                .get(&a)
                .is_some_and(|neighbors| !neighbors.contains(&b)),
        );
        let edges = self.edge_count.checked_add(add).ok_or_else(|| {
            PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
        })?;
        let bound = self.retained_bytes(self.routes.len(), edges)?;
        funding.retained(&mut self.retained_charge, bound)?;
        self.add_edge_structural_checked(funding, context, a, b)
    }

    pub(super) fn remove_edge_admitted(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        a: PartitionItemId,
        b: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let current = self.retained_bytes(self.routes.len(), self.edge_count)?;
        funding.retained(&mut self.retained_charge, current)?;
        let work = self.remove_edge_structural_checked(funding, context, a, b)?;
        let next = self.retained_bytes(self.routes.len(), self.edge_count)?;
        funding
            .retained(&mut self.retained_charge, next)
            .expect("same-lineage shrink");
        Ok(work)
    }

    pub(super) fn remove_item_admitted(
        &mut self,
        funding: &ComponentFunding<'_, '_>,
        context: &mut KernelContext<'_, '_>,
        item: PartitionItemId,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let current = self.retained_bytes(self.routes.len(), self.edge_count)?;
        funding.retained(&mut self.retained_charge, current)?;
        let work = self.remove_item_structural_checked(funding, context, item)?;
        let next = self.retained_bytes(self.routes.len(), self.edge_count)?;
        funding
            .retained(&mut self.retained_charge, next)
            .expect("same-lineage shrink");
        Ok(work)
    }

    fn retained_bytes(&self, members: usize, edges: usize) -> Result<u64, PartitionUpdateDenial> {
        members
            .checked_mul(MEMBER_SCRATCH_BYTES)
            .and_then(|bytes| bytes.checked_add(edges.checked_mul(2 * DEGREE_SCRATCH_BYTES)?))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow()))
    }
}
