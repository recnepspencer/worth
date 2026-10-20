//! Direct packet claims and retained custom structural claims share one payer.
use std::sync::{Arc, Mutex};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionResourceLease, MemoryLimitDenial, MemoryLimitLevel,
};

#[derive(Clone)]
pub(crate) struct SharedPacketScratch(Arc<Mutex<PacketScratch>>);
struct PacketScratch {
    ticket: ExecutionMemoryReservation,
    base: u64,
    direct: u64,
    structural: u64,
}
impl SharedPacketScratch {
    pub(super) fn new(
        lease: &ExecutionResourceLease<'_>,
        declared: u64,
    ) -> Result<Self, MemoryLimitDenial> {
        let base =
            (std::mem::size_of::<Mutex<PacketScratch>>() + 2 * std::mem::size_of::<usize>()) as u64;
        let total = add(base, declared)?;
        let ticket = lease.reserve_memory(total)?;
        Ok(Self(Arc::new(Mutex::new(PacketScratch {
            ticket,
            base,
            direct: 0,
            structural: 0,
        }))))
    }
    pub(super) fn reserve_direct(&self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let mut account = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = add(add(account.base, bytes)?, account.structural)?;
        grow(&mut account.ticket, total)?;
        account.direct = bytes;
        Ok(())
    }
    pub(super) fn settle_direct(&self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let mut account = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = add(add(account.base, bytes)?, account.structural)?;
        account.ticket.resize(total)?;
        account.direct = bytes;
        Ok(())
    }
    pub(crate) fn reserve_structural(&self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let mut account = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = add(add(account.base, account.direct)?, bytes)?;
        grow(&mut account.ticket, total)?;
        account.structural = account.structural.max(bytes);
        Ok(())
    }
    pub(crate) fn reserve_structural_base(&self, bytes: u64) -> Result<u64, MemoryLimitDenial> {
        let mut account = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = add(account.structural, bytes)?;
        let total = add(add(account.base, account.direct)?, next)?;
        grow(&mut account.ticket, total)?;
        account.structural = next;
        Ok(next)
    }
}
fn grow(ticket: &mut ExecutionMemoryReservation, bytes: u64) -> Result<(), MemoryLimitDenial> {
    if bytes > ticket.bytes() {
        ticket.resize(bytes)?;
    }
    Ok(())
}
fn add(charged: u64, requested: u64) -> Result<u64, MemoryLimitDenial> {
    charged
        .checked_add(requested)
        .ok_or(MemoryLimitDenial::CounterOverflow {
            requested,
            charged,
            level: MemoryLimitLevel::Declared,
        })
}
