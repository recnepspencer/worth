//! Index codec work consumes funding values, never runtime construction authority.
mod encoding;
use crate::durability::canonical_commit_artifact::borrowed_payload::PayloadCheckpoint;
use crate::durability::data::RelationalRecoveryStop as Stop;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder, ExecutionMemoryReservation,
    ExecutionRequest, WorkCeilingDenial,
};

type Reserve<'a> =
    dyn Fn(u64) -> Result<Option<ExecutionMemoryReservation>, WorkCeilingDenial> + 'a;
type Check<'a> = dyn Fn() -> Result<(), WorkCeilingDenial> + 'a;

/// The request driver supplies its own controls and reservation lineage.
pub(super) struct IndexCheckpointWork<'a> {
    pub(super) reserve: &'a Reserve<'a>,
    pub(super) check: &'a Check<'a>,
    pub(super) consult: &'a Check<'a>,
}
impl IndexCheckpointWork<'_> {
    pub(super) fn with_request<T>(
        request: &ExecutionRequest<'_, '_>,
        operation: impl FnOnce(&IndexCheckpointWork<'_>) -> T,
    ) -> T {
        operation(&IndexCheckpointWork {
            reserve: &|bytes| request.reserve_retained_memory(bytes),
            check: &|| request.check_live().map_err(WorkCeilingDenial::Stopped),
            consult: &|| request.consult(),
        })
    }
    pub(super) fn consult(&self) -> Result<(), Stop> {
        (self.consult)().map_err(|cause| Stop::Request(cause.into()))
    }
    pub(super) fn reserve(&self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Stop> {
        (self.reserve)(bytes).map_err(|cause| Stop::Request(cause.into()))
    }
    pub(super) fn array<T>(&self, count: usize) -> Result<IndexArrayBuilder<'_, T>, Stop> {
        self.consult()?;
        let bytes = std::alloc::Layout::array::<T>(count)
            .map_err(|_| super::decode_budget::overflow())?
            .size();
        let funding = self.reserve(bytes as u64)?;
        // Funding stays with this owner. SDK fixed backing still owns physical
        // layout/allocation refusal; it must not issue a second memory ticket.
        let values =
            ExecutionArrayBuilder::allocate(count, ExecutionAllocationPolicy::SystemAllocation)
                .map_err(Stop::Allocation)?;
        self.consult()?;
        Ok(IndexArrayBuilder {
            values,
            funding,
            work: self,
        })
    }
}
impl PayloadCheckpoint for IndexCheckpointWork<'_> {
    fn check_payload(&self) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        (self.check)().map_err(Into::into)
    }
}

/// Fixed SDK storage and the reservation that admitted its exact layout.
pub(crate) struct IndexArray<T> {
    values: ExecutionArray<T>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl<T> std::ops::Deref for IndexArray<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
pub(super) struct IndexArrayBuilder<'a, T> {
    values: ExecutionArrayBuilder<T>,
    funding: Option<ExecutionMemoryReservation>,
    work: &'a IndexCheckpointWork<'a>,
}
impl<T> IndexArrayBuilder<'_, T> {
    pub(super) fn len(&self) -> usize {
        self.values.len()
    }
    pub(super) fn element_count(&self) -> usize {
        self.values.element_count()
    }
    pub(super) fn push(&mut self, value: T) -> Result<(), Stop> {
        // Cancellation is checked on every write; full consultation is bounded
        // by both element count and byte granule.
        self.work.check_payload().map_err(Stop::Request)?;
        let cadence = (64 * 1024 / std::mem::size_of::<T>().max(1)).clamp(1, 32);
        if self.len().is_multiple_of(cadence) {
            self.work.consult()?;
        }
        let written = self.values.push(value).map_err(Stop::Allocation);
        if written.is_err() || self.len().is_multiple_of(cadence) {
            self.work.consult()?;
        } else {
            self.work.check_payload().map_err(Stop::Request)?;
        }
        written
    }
    pub(super) fn seal(self) -> Result<IndexArray<T>, Stop> {
        self.work.consult()?;
        let values = self.values.seal().map_err(Stop::Allocation)?;
        self.work.consult()?;
        Ok(IndexArray {
            values,
            _funding: self.funding,
        })
    }
}

impl IndexArrayBuilder<'_, u8> {
    pub(super) fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Stop> {
        for chunk in bytes.chunks(64 * 1024) {
            self.work.consult()?;
            for byte in chunk {
                self.values.push(*byte).map_err(Stop::Allocation)?;
            }
        }
        self.work.consult()
    }
}
#[cfg(test)]
pub(super) fn with_budget<T>(
    budget: &worth_execution::SerialMemoryBudget,
    operation: impl FnOnce(&IndexCheckpointWork<'_>) -> T,
) -> T {
    operation(&IndexCheckpointWork {
        reserve: &|bytes| {
            budget.reserve(bytes).map(Some).map_err(|cause| {
                WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(cause))
            })
        },
        check: &|| Ok(()),
        consult: &|| Ok(()),
    })
}
