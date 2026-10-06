//! The request memory a prepare holds for what it gathers.
//!
//! A gathered partition's bytes are held by exactly one owner at a time: the
//! prepare's reservation from before its gather until the map that computes
//! it is admitted, then the map's own admission, which charges the same
//! inputs and takes the reservation over in one ledger step.

use worth_execution::ChargedBytes;

use super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::super::WorthQueryManagedComputationResourceDenial;
use super::plan::GatheredComputationPartition;
use super::WorthQueryPartitionedComputationDenial;

/// What a prepare holds for the partitions it gathered.
pub(super) struct GatheredMemory {
    held: QueryMemoryReservation,
    declared_bytes: u64,
    /// The bytes of the partitions already gathered.
    settled: u64,
}

impl GatheredMemory {
    pub(super) fn new<Stopped>(
        execution: &QueryRequestExecution<'_>,
        declared_bytes: u64,
    ) -> Result<Self, WorthQueryPartitionedComputationDenial<Stopped>> {
        Ok(Self {
            held: execution
                .reserve(0)
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?,
            declared_bytes,
            settled: 0,
        })
    }

    /// Before one gather: the request's safe point, then the computation's
    /// declared bytes held for what the gather returns. Either refusal comes
    /// before the owner gathers anything.
    pub(super) fn before_gather<Stopped>(
        &mut self,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        execution
            .checkpoint()
            .map_err(WorthQueryPartitionedComputationDenial::from_kernel_stop)?;
        let held = self
            .settled
            .checked_add(self.declared_bytes)
            .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        self.held
            .resize(held)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)
    }

    /// After one gather: the partition's own bytes in place of the declared
    /// ones.
    pub(super) fn after_gather<Key, Gathered: ChargedBytes, Stopped>(
        &mut self,
        partition: &GatheredComputationPartition<Key, Gathered>,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let settled = u64::try_from(size_of_val(partition))
            .ok()
            .and_then(|inline| inline.checked_add(partition.additional_charged_bytes()))
            .and_then(|bytes| self.settled.checked_add(bytes))
            .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        self.held
            .resize(settled)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        self.settled = settled;
        Ok(())
    }

    /// The reservation, for the map that computes the gathered partitions to
    /// take over.
    pub(super) fn into_held(self) -> QueryMemoryReservation {
        self.held
    }
}
