//! Memory a caller holds outside any pattern: on a lease's ledger, or on the
//! serial budget a lease-free run takes from its policy.

use std::sync::{Arc, Mutex};

use worth_foundational::ExecutionRequestPolicy;

use super::lease::LeaseMemory;
use super::{ExecutionResourceLease, LeaseDenial};

/// A memory limit refused `requested` bytes. `level` names the limit, the
/// innermost one that refused, and `admitted` is the room that level left.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryLimitDenial {
    pub requested: u64,
    pub admitted: u64,
    pub level: MemoryLimitLevel,
}

/// Which limit refused a memory reservation. Limits are checked innermost
/// first, so a charge over its own policy reads the same however much other
/// requests hold, and only the process's refusal can clear when they release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryLimitLevel {
    /// The policy of the lease the charge is on at `ancestor` 0, its
    /// parent's at 1, and so on outward. A lease-free run's serial budget is
    /// its own policy, at 0.
    Policy { ancestor: u32 },
    /// The process's memory, which every request shares.
    Process,
    /// The bytes the pattern itself declared for what it holds.
    Declared,
}

/// The memory limit of a run with a policy and no lease: the policy's charged
/// memory, held on the calling thread. Every lease-free pattern inside
/// [`crate::ExecutionWorkCeiling::run_serial`] reserves its framework bytes,
/// at one worker, against it, beside what the caller reserves here.
#[derive(Clone, Debug)]
pub struct SerialMemoryBudget {
    ledger: Arc<SerialLedger>,
}

#[derive(Debug)]
struct SerialLedger {
    ceiling: u64,
    charged: Mutex<u64>,
}

/// Bytes held against a lease's ledger or a serial budget until dropped.
#[derive(Debug)]
pub struct ExecutionMemoryReservation {
    held: Held,
}

#[derive(Debug)]
enum Held {
    Lease(LeaseMemory),
    Serial {
        ledger: Arc<SerialLedger>,
        bytes: u64,
    },
}

impl SerialMemoryBudget {
    pub fn from_policy(policy: &ExecutionRequestPolicy) -> Self {
        Self {
            ledger: Arc::new(SerialLedger {
                ceiling: policy.budget().charged_memory_bytes(),
                charged: Mutex::new(0),
            }),
        }
    }

    pub fn reserve(&self, bytes: u64) -> Result<ExecutionMemoryReservation, MemoryLimitDenial> {
        self.ledger.resize(0, bytes)?;
        Ok(ExecutionMemoryReservation {
            held: Held::Serial {
                ledger: Arc::clone(&self.ledger),
                bytes,
            },
        })
    }
}

impl SerialLedger {
    fn resize(&self, from: u64, to: u64) -> Result<(), MemoryLimitDenial> {
        let mut charged = self
            .charged
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let others = *charged - from;
        let admitted = self.ceiling.saturating_sub(others);
        if to > admitted {
            return Err(MemoryLimitDenial {
                requested: to,
                admitted,
                level: MemoryLimitLevel::Policy { ancestor: 0 },
            });
        }
        *charged = others + to;
        Ok(())
    }
}

impl ExecutionMemoryReservation {
    pub(super) const fn lease(memory: LeaseMemory) -> Self {
        Self {
            held: Held::Lease(memory),
        }
    }

    pub fn bytes(&self) -> u64 {
        match &self.held {
            Held::Lease(memory) => memory.bytes(),
            Held::Serial { bytes, .. } => *bytes,
        }
    }

    /// Holds a run's `bytes` in place of what was held. Lease memory moves
    /// onto the run's lease's lineage; serial memory stays on its budget,
    /// which must be the run's. Either way the move is one ledger step, and a
    /// refusal keeps what was held.
    pub(crate) fn take_over(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        serial: Option<&SerialMemoryBudget>,
        bytes: u64,
    ) -> Result<(), LeaseDenial> {
        match (&mut self.held, lease) {
            (Held::Lease(memory), Some(lease)) => memory.rebind(lease, bytes),
            (
                Held::Serial {
                    ledger,
                    bytes: held,
                },
                None,
            ) if serial.is_some_and(|budget| Arc::ptr_eq(ledger, &budget.ledger)) => {
                ledger
                    .resize(*held, bytes)
                    .map_err(LeaseDenial::MemoryExhausted)?;
                *held = bytes;
                Ok(())
            }
            _ => Err(LeaseDenial::UnrelatedNestedLease),
        }
    }

    /// Holds `bytes` instead. A refused resize keeps what was held.
    pub fn resize(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        match &mut self.held {
            Held::Lease(memory) => memory.resize(bytes),
            Held::Serial {
                ledger,
                bytes: held,
            } => {
                ledger.resize(*held, bytes)?;
                *held = bytes;
                Ok(())
            }
        }
    }
}

impl Drop for ExecutionMemoryReservation {
    fn drop(&mut self) {
        if let Held::Serial { ledger, bytes } = &self.held {
            let _ = ledger.resize(*bytes, 0);
        }
    }
}
