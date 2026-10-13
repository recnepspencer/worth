//! Request-owned input storage, admitted before planning or cache construction.
use std::alloc::Layout;
use std::sync::Mutex;

use worth_execution::{ExecutionLeaseStatus, ExecutionMemoryReservation, ExecutionResourceLease};

use crate::execution::{PacketBudgetDenial, RelationalExecutionDenialCause as Cause};

pub(super) struct CandidateInputAllocation {
    state: Mutex<AllocationState>,
    status: Option<ExecutionLeaseStatus>,
}

pub(crate) struct CandidateInputTemporary<'a> {
    allocation: &'a CandidateInputAllocation,
    bytes: u64,
}

impl CandidateInputTemporary<'_> {
    pub(crate) fn claim(&mut self, bytes: u64) -> bool {
        if !self.allocation.claim(bytes) {
            return false;
        }
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .expect("admitted temporary input bytes");
        true
    }
}

impl Drop for CandidateInputTemporary<'_> {
    fn drop(&mut self) {
        self.allocation.release(self.bytes);
    }
}

struct AllocationState {
    hold: Option<ExecutionMemoryReservation>,
    failure: Option<PacketBudgetDenial>,
}

impl CandidateInputAllocation {
    pub(super) fn temporary(&self) -> CandidateInputTemporary<'_> {
        CandidateInputTemporary {
            allocation: self,
            bytes: 0,
        }
    }
    pub(super) fn is_leased(&self) -> bool {
        self.status.is_some()
    }
    pub(super) fn new(
        lease: Option<&ExecutionResourceLease<'_>>,
        bytes: u64,
    ) -> Result<Self, PacketBudgetDenial> {
        if let Some(lease) = lease {
            let status = lease.status();
            if status.is_cancelled() {
                return Err(PacketBudgetDenial::ReadRequest(Cause::Cancelled));
            }
            if status.deadline_elapsed() {
                return Err(PacketBudgetDenial::ReadRequest(Cause::DeadlineElapsed));
            }
        }
        let hold = lease
            .map(|lease| lease.reserve_memory(bytes).map_err(memory_failure))
            .transpose()?;
        Ok(Self {
            state: Mutex::new(AllocationState {
                hold,
                failure: None,
            }),
            status: lease.map(ExecutionResourceLease::status),
        })
    }

    pub(super) fn claim(&self, bytes: u64) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.failure.is_some() {
            return false;
        }
        let stopped = self.status.as_ref().and_then(|status| {
            if status.is_cancelled() {
                Some(Cause::Cancelled)
            } else if status.deadline_elapsed() {
                Some(Cause::DeadlineElapsed)
            } else {
                None
            }
        });
        if let Some(cause) = stopped {
            state.failure = Some(PacketBudgetDenial::ReadRequest(cause));
            return false;
        }
        let Some(hold) = &mut state.hold else {
            return true;
        };
        let Some(total) = hold.bytes().checked_add(bytes) else {
            state.failure = Some(PacketBudgetDenial::ReadRequest(Cause::ChargedBytesOverflow));
            return false;
        };
        if let Err(denial) = hold.resize(total) {
            state.failure = Some(memory_failure(denial));
            return false;
        }
        true
    }

    /// Release only storage already destroyed by its owner.
    pub(super) fn release(&self, bytes: u64) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(hold) = &mut state.hold {
            let remaining = hold.bytes().checked_sub(bytes).expect("owned input charge");
            hold.resize(remaining)
                .expect("shrinking an input reservation");
        }
    }

    pub(super) fn claim_array<T>(&self, length: usize) -> bool {
        match Layout::array::<T>(length) {
            Ok(layout) => self.claim(layout.size() as u64),
            Err(_) => self.refuse_overflow(),
        }
    }

    pub(super) fn claim_arc_slice<T>(&self, length: usize) -> bool {
        match Layout::array::<T>(length).and_then(|layout| {
            Layout::new::<[usize; 2]>()
                .extend(layout)
                .map(|(layout, _)| layout.pad_to_align())
        }) {
            Ok(layout) => self.claim(layout.size() as u64),
            Err(_) => self.refuse_overflow(),
        }
    }

    fn refuse_overflow(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .failure
            .get_or_insert(PacketBudgetDenial::ReadRequest(Cause::ChargedBytesOverflow));
        false
    }

    pub(super) fn failure(&self) -> Option<PacketBudgetDenial> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .failure
    }
}

pub(super) fn arc_bytes<T>() -> u64 {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .expect("input owner layout fits")
        .0
        .pad_to_align()
        .size() as u64
}

fn memory_failure(denial: worth_execution::MemoryLimitDenial) -> PacketBudgetDenial {
    PacketBudgetDenial::ReadRequest(crate::execution::denial::lease_denial(
        worth_execution::LeaseDenial::MemoryExhausted(denial),
    ))
}
