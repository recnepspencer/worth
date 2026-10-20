//! Fixed packet declarations and live native allocation tickets share one claim door.
use super::PacketBudgetDenial;
mod scratch;
pub(crate) use scratch::SharedPacketScratch;
use std::cell::{Cell, RefCell};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext, MapKernelFailure,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum PacketMemory {
    Declared { scratch: u64, result: u64 },
    Tracked,
}
impl PacketMemory {
    fn scratch(self) -> Option<u64> {
        match self {
            Self::Declared { scratch, .. } => Some(scratch),
            Self::Tracked => None,
        }
    }
    fn result(self) -> Option<u64> {
        match self {
            Self::Declared { result, .. } => Some(result),
            Self::Tracked => None,
        }
    }
}
struct TrackedMemory {
    scratch: SharedPacketScratch,
    result: RefCell<ExecutionMemoryReservation>,
}
pub(crate) struct PacketKernelContext<'a, 'b, 'c> {
    meter: &'a mut MapKernelContext<'b, 'c>,
    memory: PacketMemory,
    tracking: Option<TrackedMemory>,
    scratch_bytes: u64,
    temporary_scratch: Cell<u64>,
    result_bytes: u64,
}

impl PacketKernelContext<'_, '_, '_> {
    pub(crate) fn remaining_work(&self) -> Option<u64> {
        self.meter.remaining_work()
    }
    pub(crate) fn remaining_scratch(&self) -> Option<u64> {
        self.memory
            .scratch()
            .map(|limit| limit.saturating_sub(self.scratch_bytes))
    }
    pub(crate) fn structural_memory(&self) -> Option<SharedPacketScratch> {
        self.tracking
            .as_ref()
            .filter(|_| matches!(self.memory, PacketMemory::Tracked))
            .map(|tracking| tracking.scratch.clone())
    }

    pub(crate) fn account_completed_work(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.meter
            .account_completed_work(units)
            .map_err(MapKernelFailure::Stop)
    }
    pub(crate) fn checkpoint(
        &mut self,
        units: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        self.meter.checkpoint(units).map_err(MapKernelFailure::Stop)
    }

    pub(crate) fn claim_scratch(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let next = self.scratch_bytes.checked_add(bytes).ok_or_else(|| {
            self.claim_overflow(
                bytes,
                self.scratch_bytes,
                MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded),
            )
        })?;
        if self.memory.scratch().is_some_and(|limit| next > limit) {
            return Err(MapKernelFailure::Domain(
                PacketBudgetDenial::ScratchCapacityExceeded,
            ));
        }
        let total = next
            .checked_add(self.temporary_scratch.get())
            .ok_or_else(|| {
                self.claim_overflow(
                    bytes,
                    next,
                    MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded),
                )
            })?;
        self.admit_scratch(total)?;
        self.scratch_bytes = next;
        Ok(())
    }

    /// The source has dropped its conservative preclone shape or its borrowed
    /// iterator; settle this live temporary while retaining every direct/custom claim.
    pub(crate) fn settle_temporary_scratch(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        if !matches!(self.memory, PacketMemory::Tracked) {
            return self.check_scratch_peak(bytes);
        }
        let total = self.scratch_bytes.checked_add(bytes).ok_or_else(|| {
            self.claim_overflow(
                bytes,
                self.scratch_bytes,
                MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded),
            )
        })?;
        if let Some(tracking) = &self.tracking {
            tracking.scratch.settle_direct(total).map_err(memory_stop)?;
        }
        self.temporary_scratch.set(bytes);
        Ok(())
    }

    pub(crate) fn check_scratch_peak(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let peak = self.scratch_bytes.checked_add(bytes).ok_or_else(|| {
            self.claim_overflow(
                bytes,
                self.scratch_bytes,
                MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded),
            )
        })?;
        if self.memory.scratch().is_some_and(|limit| peak > limit) {
            return Err(MapKernelFailure::Domain(
                PacketBudgetDenial::ScratchCapacityExceeded,
            ));
        }
        self.admit_scratch(peak)?;
        Ok(())
    }

    pub(crate) fn claim_result(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let next = self.result_bytes.checked_add(bytes).ok_or_else(|| {
            self.claim_overflow(
                bytes,
                self.result_bytes,
                MapKernelFailure::ResultCapacityExceeded,
            )
        })?;
        if self.memory.result().is_some_and(|limit| next > limit) {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        self.admit_result(next)?;
        self.result_bytes = next;
        Ok(())
    }

    pub(crate) fn check_result_peak(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        let peak = self.result_bytes.checked_add(bytes).ok_or_else(|| {
            self.claim_overflow(
                bytes,
                self.result_bytes,
                MapKernelFailure::ResultCapacityExceeded,
            )
        })?;
        if self.memory.result().is_some_and(|limit| peak > limit) {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        self.admit_result(peak)?;
        Ok(())
    }

    pub(crate) fn ensure_result_total(
        &mut self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        if bytes > self.result_bytes {
            self.claim_result(bytes - self.result_bytes)?;
        }
        Ok(())
    }
}

impl<'a, 'b, 'c> PacketKernelContext<'a, 'b, 'c> {
    pub(super) fn declared(meter: &'a mut MapKernelContext<'b, 'c>, memory: PacketMemory) -> Self {
        Self {
            meter,
            memory,
            tracking: None,
            scratch_bytes: 0,
            temporary_scratch: Cell::new(0),
            result_bytes: 0,
        }
    }
    pub(super) fn tracked(
        meter: &'a mut MapKernelContext<'b, 'c>,
        memory: PacketMemory,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<Self, MapKernelFailure<PacketBudgetDenial>> {
        let (scratch_bytes, result_bytes) = match memory {
            PacketMemory::Declared { scratch, result } => (scratch, result),
            PacketMemory::Tracked => (0, 0),
        };
        let scratch = SharedPacketScratch::new(lease, scratch_bytes).map_err(memory_stop)?;
        let result = lease.reserve_memory(result_bytes).map_err(memory_stop)?;
        Ok(Self {
            meter,
            memory,
            tracking: Some(TrackedMemory {
                scratch,
                result: RefCell::new(result),
            }),
            scratch_bytes: 0,
            temporary_scratch: Cell::new(0),
            result_bytes: 0,
        })
    }
    fn claim_overflow(
        &self,
        bytes: u64,
        charged: u64,
        declared: MapKernelFailure<PacketBudgetDenial>,
    ) -> MapKernelFailure<PacketBudgetDenial> {
        if matches!(self.memory, PacketMemory::Tracked) {
            memory_stop(worth_execution::MemoryLimitDenial::CounterOverflow {
                requested: bytes,
                charged,
                level: worth_execution::MemoryLimitLevel::Declared,
            })
        } else {
            declared
        }
    }
    fn admit_scratch(&self, bytes: u64) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        if let Some(tracking) = &self.tracking {
            tracking
                .scratch
                .reserve_direct(bytes)
                .map_err(memory_stop)?;
        }
        Ok(())
    }
    fn admit_result(&self, bytes: u64) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
        if let Some(tracking) = &self.tracking {
            let mut ticket = tracking.result.borrow_mut();
            if bytes > ticket.bytes() {
                ticket.resize(bytes).map_err(memory_stop)?;
            }
        }
        Ok(())
    }

    pub(super) fn capture_result(
        mut self,
        bytes: u64,
    ) -> Result<ExecutionMemoryReservation, MapKernelFailure<PacketBudgetDenial>> {
        self.ensure_result_total(bytes)?;
        let tracking = self
            .tracking
            .take()
            .expect("owner carriage holds real tickets");
        let mut ticket = tracking.result.into_inner();
        ticket.resize(bytes).map_err(memory_stop)?;
        Ok(ticket)
    }
}
fn memory_stop(denial: worth_execution::MemoryLimitDenial) -> MapKernelFailure<PacketBudgetDenial> {
    MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
        super::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(denial)),
    ))
}
