//! Bounded rolling residency windows for scans larger than the working set.
//!
//! A window holds operation-scoped borrows for at most `capacity` regions.
//! Advancing releases the oldest borrow before demanding the next region, so
//! a scan never pins the whole root, and a saturated working set first sheds
//! the window's own borrows. `MemoryExhausted` escapes only when one region
//! cannot fit with the window empty.
use super::RelationalResidencyLease;
use super::{Full, Head, RelationalRegionRetention, RelationalResidencyDemandFailure as Failure};
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::VecDeque;
use std::num::NonZeroUsize;

/// Residency level a window borrows, through the non-retaining root doors.
pub trait RollingWindowLevel: Sized {
    fn borrow(
        retention: &RelationalRegionRetention,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Self>, Failure>;
}

impl RollingWindowLevel for Head {
    fn borrow(
        retention: &RelationalRegionRetention,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Self>, Failure> {
        retention.borrow_head(execution, limit)
    }
}

impl RollingWindowLevel for Full {
    fn borrow(
        retention: &RelationalRegionRetention,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Self>, Failure> {
        retention.borrow_full(execution, limit)
    }
}

pub struct RollingResidencyWindow<'root, Level, Regions>
where
    Regions: Iterator<Item = (PartitionId, &'root RelationalRegionRetention)>,
{
    regions: Regions,
    capacity: NonZeroUsize,
    limit: u64,
    held: VecDeque<(PartitionId, RelationalResidencyLease<Level>)>,
}

fn memory_exhausted(failure: &Failure) -> bool {
    matches!(failure, Failure::StorageUnavailable(unavailable)
        if unavailable.cause == Cause::MemoryExhausted)
}

impl<'root, Level, Regions> RollingResidencyWindow<'root, Level, Regions>
where
    Level: RollingWindowLevel,
    Regions: Iterator<Item = (PartitionId, &'root RelationalRegionRetention)>,
{
    pub fn new(
        regions: impl IntoIterator<IntoIter = Regions>,
        capacity: NonZeroUsize,
        limit: u64,
    ) -> Self {
        Self {
            regions: regions.into_iter(),
            capacity,
            limit,
            held: VecDeque::with_capacity(capacity.get()),
        }
    }

    /// Borrows currently held; never more than the window capacity.
    pub fn held(&self) -> usize {
        self.held.len()
    }

    /// Borrow the next region, returning `None` once every region was visited.
    pub fn advance(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<Result<(PartitionId, &RelationalResidencyLease<Level>), Failure>> {
        let (id, retention) = self.regions.next()?;
        if self.held.len() == self.capacity.get() {
            self.held.pop_front();
        }
        let lease = match Level::borrow(retention, execution, self.limit) {
            Err(failure) if memory_exhausted(&failure) && !self.held.is_empty() => {
                // Shed this window's own borrows; a lone region must still fit.
                self.held.clear();
                Level::borrow(retention, execution, self.limit)
            }
            other => other,
        };
        Some(lease.map(|lease| {
            self.held.push_back((id, lease));
            let (id, lease) = self.held.back().expect("just pushed");
            (*id, lease)
        }))
    }
}
