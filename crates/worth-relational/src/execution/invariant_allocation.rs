//! Live invariant allocations draw from the exact packet-dispatch lease.
use super::PacketBudgetDenial;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial};

struct Allocation {
    hold: ExecutionMemoryReservation,
    base_bytes: u64,
    initial_room: u64,
    claimed: u64,
    failure: Option<PacketBudgetDenial>,
}

impl Allocation {
    fn overflow(&mut self) -> PacketBudgetDenial {
        let failure = PacketBudgetDenial::ReadRequest(
            super::RelationalExecutionDenialCause::ChargedBytesOverflow,
        );
        self.failure = Some(failure);
        failure
    }
    fn reserve(&mut self, total: u64) -> Result<(), PacketBudgetDenial> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        let total = total
            .checked_add(self.base_bytes)
            .ok_or_else(|| self.overflow())?;
        if total <= self.hold.bytes() {
            return Ok(());
        }
        self.hold.resize(total).map_err(|denial| {
            let failure = PacketBudgetDenial::ReadRequest(super::denial::lease_denial(
                LeaseDenial::MemoryExhausted(denial),
            ));
            self.failure = Some(failure);
            failure
        })
    }
    fn claim(&mut self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        let total = self
            .claimed
            .checked_add(bytes)
            .ok_or_else(|| self.overflow())?;
        self.reserve(total)?;
        self.claimed = total;
        Ok(())
    }
    fn peak(&mut self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        let total = self
            .claimed
            .checked_add(bytes)
            .ok_or_else(|| self.overflow())?;
        self.reserve(total)
    }
}

/// Shared by authored kernel claims and its structural view allocations.
#[derive(Clone)]
pub(crate) struct InvariantScratchAdmission(Arc<Mutex<Allocation>>);

impl InvariantScratchAdmission {
    pub(crate) fn new(lease: &ExecutionResourceLease<'_>) -> Result<Self, PacketBudgetDenial> {
        let base_bytes =
            (std::mem::size_of::<Mutex<Allocation>>() + 2 * std::mem::size_of::<usize>()) as u64;
        let hold = lease.reserve_memory(base_bytes).map_err(|denial| {
            PacketBudgetDenial::ReadRequest(super::denial::lease_denial(
                LeaseDenial::MemoryExhausted(denial),
            ))
        })?;
        Ok(Self(Arc::new(Mutex::new(Allocation {
            hold,
            base_bytes,
            initial_room: lease.available_memory_bytes(),
            claimed: 0,
            failure: None,
        }))))
    }
    pub(crate) fn claim(&self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .claim(bytes)
    }
    pub(crate) fn check_peak(&self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .peak(bytes)
    }
    pub(crate) fn failure(&self) -> Option<PacketBudgetDenial> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .failure
    }
    pub(crate) fn remaining_hint(&self) -> u64 {
        let allocation = self.0.lock().unwrap_or_else(|error| error.into_inner());
        allocation
            .initial_room
            .saturating_sub(allocation.hold.bytes())
    }
}

pub(crate) struct InvariantPacketAllocations {
    scratch: InvariantScratchAdmission,
    result: Allocation,
}

impl InvariantPacketAllocations {
    pub(crate) fn new(lease: &ExecutionResourceLease<'_>) -> Result<Self, PacketBudgetDenial> {
        let allocation = |base_bytes| {
            lease
                .reserve_memory(base_bytes)
                .map(|hold| Allocation {
                    hold,
                    base_bytes,
                    initial_room: lease.available_memory_bytes(),
                    claimed: 0,
                    failure: None,
                })
                .map_err(|denial| {
                    PacketBudgetDenial::ReadRequest(super::denial::lease_denial(
                        LeaseDenial::MemoryExhausted(denial),
                    ))
                })
        };
        Ok(Self {
            scratch: InvariantScratchAdmission::new(lease)?,
            result: allocation(0)?,
        })
    }
    pub(crate) fn scratch(&self) -> InvariantScratchAdmission {
        self.scratch.clone()
    }
    pub(crate) fn claim_result(&mut self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        self.result.claim(bytes)
    }
    pub(crate) fn check_result_peak(&mut self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        self.result.peak(bytes)
    }
    pub(crate) fn ensure_result_total(&mut self, bytes: u64) -> Result<(), PacketBudgetDenial> {
        if bytes <= self.result.hold.bytes() {
            // Construction is complete. Transfer only the returned image's
            // actual capacity, releasing temporary formatting/evaluation space.
            return self.result.hold.resize(bytes).map_err(|denial| {
                PacketBudgetDenial::ReadRequest(super::denial::lease_denial(
                    LeaseDenial::MemoryExhausted(denial),
                ))
            });
        }
        Err(PacketBudgetDenial::ReadRequest(
            super::RelationalExecutionDenialCause::DeclaredMemoryExhausted {
                requested: bytes,
                admitted: self.result.hold.bytes(),
            },
        ))
    }
    pub(crate) fn into_result_custody(self) -> ExecutionMemoryReservation {
        self.result.hold
    }
}
