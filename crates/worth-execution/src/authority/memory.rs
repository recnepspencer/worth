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
    ledgers: Arc<[Arc<SerialLedger>]>,
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
        budget: SerialMemoryBudget,
        bytes: u64,
    },
}

impl SerialMemoryBudget {
    pub fn limit(&self) -> u64 {
        self.ledgers[0].ceiling
    }

    /// Uses only the policy memory limit; the work ceiling belongs to leased requests.
    pub fn from_policy(policy: &ExecutionRequestPolicy) -> Self {
        Self::new(policy.budget().charged_memory_bytes())
    }

    /// Holds the owner's required policy memory limit; serial work is metered, not bounded.
    pub fn new(charged_memory_bytes: u64) -> Self {
        Self {
            ledgers: Arc::from([Arc::new(SerialLedger {
                ceiling: charged_memory_bytes,
                charged: Mutex::new(0),
            })]),
        }
    }

    pub fn reserve(&self, bytes: u64) -> Result<ExecutionMemoryReservation, MemoryLimitDenial> {
        self.resize(0, bytes)?;
        Ok(ExecutionMemoryReservation {
            held: Held::Serial {
                budget: self.clone(),
                bytes,
            },
        })
    }
}

impl SerialMemoryBudget {
    /// Retain the child's own limit and draw every reservation from its parent.
    pub(crate) fn within_parent(&self, parent: &Self) -> Self {
        let mut ledgers = self.ledgers.to_vec();
        for ledger in parent.ledgers.iter() {
            if !ledgers.iter().any(|own| Arc::ptr_eq(own, ledger)) {
                ledgers.push(Arc::clone(ledger));
            }
        }
        Self {
            ledgers: ledgers.into(),
        }
    }

    fn same_lineage(&self, other: &Self) -> bool {
        self.ledgers.len() == other.ledgers.len()
            && self
                .ledgers
                .iter()
                .zip(other.ledgers.iter())
                .all(|(a, b)| Arc::ptr_eq(a, b))
    }

    fn resize(&self, from: u64, to: u64) -> Result<(), MemoryLimitDenial> {
        // Requests can share budgets across threads. Address order keeps overlapping
        // lineages deadlock-free; refusal order remains innermost policy first.
        let mut order: Vec<_> = self.ledgers.iter().enumerate().collect();
        order.sort_unstable_by_key(|(_, ledger)| Arc::as_ptr(ledger));
        let mut charges: Vec<_> = order
            .iter()
            .map(|(index, ledger)| {
                (
                    *index,
                    ledger
                        .charged
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                )
            })
            .collect();
        for (ancestor, ledger) in self.ledgers.iter().enumerate() {
            let charge = charges
                .iter()
                .find(|(index, _)| *index == ancestor)
                .unwrap();
            let admitted = ledger.ceiling.saturating_sub(*charge.1 - from);
            if to > admitted {
                return Err(MemoryLimitDenial {
                    requested: to,
                    admitted,
                    level: MemoryLimitLevel::Policy {
                        ancestor: ancestor as u32,
                    },
                });
            }
        }
        for (_, charge) in &mut charges {
            **charge = **charge - from + to;
        }
        Ok(())
    }
}

impl ExecutionMemoryReservation {
    pub fn reserve_in_scope(
        lease: Option<&ExecutionResourceLease<'_>>,
        bytes: u64,
    ) -> Result<Self, LeaseDenial> {
        if let Some(lease) = lease {
            return lease
                .reserve_memory(bytes)
                .map_err(LeaseDenial::MemoryExhausted);
        }
        crate::backend::active_serial_memory()
            .ok_or(LeaseDenial::NoActiveExecutionScope)?
            .reserve(bytes)
            .map_err(LeaseDenial::MemoryExhausted)
    }

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
                    budget,
                    bytes: held,
                },
                None,
            ) if serial.is_some_and(|active| budget.same_lineage(active)) => {
                budget
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
                budget,
                bytes: held,
            } => {
                budget.resize(*held, bytes)?;
                *held = bytes;
                Ok(())
            }
        }
    }
}

impl Drop for ExecutionMemoryReservation {
    fn drop(&mut self) {
        if let Held::Serial { budget, bytes } = &self.held {
            let _ = budget.resize(*bytes, 0);
        }
    }
}
