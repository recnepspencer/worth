//! Structural subclaims retain the exact original packet payer.
use worth_execution::{ExecutionMemoryReservation, MemoryLimitDenial, MemoryLimitLevel};
pub(super) enum PreparationMemory {
    Owned {
        payer: ExecutionMemoryReservation,
        base: u64,
    },
    Packet {
        payer: crate::execution::SharedPacketScratch,
        base: u64,
    },
}
impl PreparationMemory {
    pub(super) fn reserve(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let base = match self {
            Self::Owned { base, .. } | Self::Packet { base, .. } => *base,
        };
        let total = base
            .checked_add(bytes)
            .ok_or(MemoryLimitDenial::CounterOverflow {
                requested: bytes,
                charged: base,
                level: MemoryLimitLevel::Declared,
            })?;
        match self {
            Self::Owned { payer, .. } => grow(payer, total),
            Self::Packet { payer, .. } => payer.reserve_structural(total),
        }
    }
}
fn grow(ticket: &mut ExecutionMemoryReservation, bytes: u64) -> Result<(), MemoryLimitDenial> {
    if bytes > ticket.bytes() {
        ticket.resize(bytes)?;
    }
    Ok(())
}

use super::{CustomInvariantWorkMeter, CustomPreparationStop};
use std::sync::atomic::Ordering;
impl CustomInvariantWorkMeter {
    pub(crate) fn try_claim_bytes(&self, bytes: u64) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        if !self.safe_point() {
            return false;
        }
        let mut memory = leased
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = leased.claimed_memory.load(Ordering::Relaxed);
        let Some(next) = current.checked_add(bytes) else {
            drop(memory);
            self.stop(CustomPreparationStop::MemoryAdmission(
                worth_execution::MemoryLimitDenial::CounterOverflow {
                    requested: bytes,
                    charged: current,
                    level: worth_execution::MemoryLimitLevel::Declared,
                },
            ));
            return false;
        };
        if leased.memory_ceiling.is_some_and(|ceiling| next > ceiling) {
            drop(memory);
            self.stop(CustomPreparationStop::MemoryExhausted);
            return false;
        }
        if let Some(ticket) = memory.as_mut() {
            {
                if let Err(denial) = ticket.reserve(next) {
                    drop(memory);
                    self.stop(CustomPreparationStop::MemoryAdmission(denial));
                    return false;
                }
            }
        }
        leased.claimed_memory.store(next, Ordering::Relaxed);
        true
    }

    pub(super) fn check_claim_peak(&self, bytes: u64) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        if !self.safe_point() {
            return false;
        }
        let mut memory = leased
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = leased.claimed_memory.load(Ordering::Relaxed);
        let Some(peak) = current.checked_add(bytes) else {
            drop(memory);
            self.stop(CustomPreparationStop::MemoryAdmission(
                MemoryLimitDenial::CounterOverflow {
                    requested: bytes,
                    charged: current,
                    level: MemoryLimitLevel::Declared,
                },
            ));
            return false;
        };
        if leased.memory_ceiling.is_some_and(|ceiling| peak > ceiling) {
            drop(memory);
            self.stop(CustomPreparationStop::MemoryExhausted);
            return false;
        }
        if let Some(ticket) = memory.as_mut() {
            {
                if let Err(denial) = ticket.reserve(peak) {
                    drop(memory);
                    self.stop(CustomPreparationStop::MemoryAdmission(denial));
                    return false;
                }
            }
        }
        true
    }
}
