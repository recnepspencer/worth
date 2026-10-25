use super::*;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum Head {}
#[derive(Debug)]
pub(crate) enum Full {}

/// Clones and releases are counted on the cell, including when an operation
/// retains a view after the root that supplied it has been released.
#[derive(Debug)]
pub(crate) struct RelationalResidencyLease<Level> {
    cell: Arc<RelationalRegionResidency>,
    region: Arc<ResidentRegion>,
    level: PhantomData<Level>,
}

impl<Level> RelationalResidencyLease<Level> {
    pub(super) fn accepted(
        cell: Arc<RelationalRegionResidency>,
        region: Arc<ResidentRegion>,
    ) -> Self {
        Self {
            cell,
            region,
            level: PhantomData,
        }
    }

    pub(crate) fn head_view(&self) -> RelationalRegionHeadView<'_> {
        RelationalRegionHeadView::new(&self.region)
    }
}

impl RelationalResidencyLease<Full> {
    pub(crate) fn residency_allocation_bytes(&self) -> u64 {
        (std::mem::size_of::<RelationalRegionResidency>()
            + std::mem::size_of::<ResidentRegion>()
            + 4 * std::mem::size_of::<usize>()) as u64
    }

    pub(crate) fn full_view(&self) -> RelationalRegionFullView<'_> {
        RelationalRegionFullView::new(&self.region)
    }

    pub(crate) fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell) == 1 && Arc::strong_count(&self.region) == 2
    }
}

impl AsRef<crate::storage::overlay::PartitionState> for RelationalResidencyLease<Full> {
    fn as_ref(&self) -> &crate::storage::overlay::PartitionState {
        self.full_view().partition()
    }
}

impl std::ops::Deref for RelationalResidencyLease<Full> {
    type Target = crate::storage::overlay::PartitionState;
    fn deref(&self) -> &Self::Target {
        self.full_view().partition()
    }
}

impl<Level> Clone for RelationalResidencyLease<Level> {
    fn clone(&self) -> Self {
        self.cell
            .leases
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect("a resident lease count fits the address space");
        Self::accepted(self.cell.clone(), self.region.clone())
    }
}

impl<Level> Drop for RelationalResidencyLease<Level> {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}
