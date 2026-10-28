//! Memory a caller holds outside any pattern: on a lease's ledger, or on the
//! serial budget a lease-free run takes from its policy.

use std::sync::{Arc, Mutex};

use worth_foundational::ExecutionRequestPolicy;

use super::lease::LeaseMemory;
use super::{ExecutionResourceLease, LeaseDenial};
mod ledger_issue_order;
mod observation;
mod result;
use ledger_issue_order::LedgerIssueOrder;
pub use result::ExecutionResultMemory;

/// A finite ceiling refusal or an unrepresentable live charge, at its original owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryLimitDenial {
    /// The innermost supplied ceiling left less room than requested.
    Capacity {
        requested: u64,
        admitted: u64,
        level: MemoryLimitLevel,
    },
    /// The held bytes plus this request do not fit the accounting counter.
    CounterOverflow {
        requested: u64,
        charged: u64,
        level: MemoryLimitLevel,
    },
}

impl MemoryLimitDenial {
    pub(crate) fn refused(requested: u64, admitted: u64, level: MemoryLimitLevel) -> Self {
        crate::backend::request_refusal(Self::Capacity {
            requested,
            admitted,
            level,
        })
    }

    pub(crate) fn overflow(requested: u64, charged: u64, level: MemoryLimitLevel) -> Self {
        crate::backend::request_stop(Self::CounterOverflow {
            requested,
            charged,
            level,
        })
    }

    pub fn level(self) -> MemoryLimitLevel {
        match self {
            Self::Capacity { level, .. } | Self::CounterOverflow { level, .. } => level,
        }
    }
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
    issue_order: LedgerIssueOrder,
    ceiling: Option<u64>,
    charged: Mutex<u64>,
}

/// Bytes held against a lease's ledger or a serial budget until dropped.
///
/// Tickets cannot be constructed without live execution admission.
///
/// ```compile_fail
/// use worth_execution::ExecutionMemoryReservation;
/// let ticket = ExecutionMemoryReservation {};
/// ```
///
/// One charge cannot be cloned into independent owners.
///
/// ```compile_fail
/// use worth_execution::ExecutionMemoryReservation;
/// fn duplicate(ticket: &ExecutionMemoryReservation) -> ExecutionMemoryReservation {
///     ticket.clone()
/// }
/// ```

#[derive(Debug)]
pub struct ExecutionMemoryReservation {
    held: Held,
    activity: Option<crate::backend::MemoryActivityGuard>,
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
    /// Bytes currently held against this budget's own policy, including descendants.
    /// This observation grants no reservation or admission authority.
    pub fn charged_memory_bytes(&self) -> u64 {
        *self.ledgers[0]
            .charged
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// This budget's own policy ceiling, without ancestor ceilings or live holds.
    /// Use `available_memory_bytes` on the active lineage for admission planning.
    pub fn limit(&self) -> Option<u64> {
        self.ledgers[0].ceiling
    }

    /// Uses only the policy memory limit; the work ceiling belongs to leased requests.
    pub fn from_policy(policy: &ExecutionRequestPolicy) -> Self {
        Self::new(
            policy
                .accounting()
                .and_then(|budget| budget.charged_memory_bytes()),
        )
    }

    /// Retains an optional caller ceiling; every live byte remains accounted.
    pub fn new(charged_memory_bytes: Option<u64>) -> Self {
        Self {
            ledgers: Arc::from([Arc::new(SerialLedger {
                issue_order: LedgerIssueOrder::issue(),
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
            activity: None,
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
        // Requests can share budgets across threads. Issue order keeps overlapping
        // lineages deadlock-free; refusal order remains innermost policy first.
        let order = self.lock_order();
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
            let charged = *charge.1 - from;
            let level = MemoryLimitLevel::Policy {
                ancestor: ancestor as u32,
            };
            if let Some(ceiling) = ledger.ceiling {
                let admitted = ceiling.saturating_sub(charged);
                if to > admitted {
                    return Err(MemoryLimitDenial::refused(to, admitted, level));
                }
            }
            charged
                .checked_add(to)
                .ok_or_else(|| MemoryLimitDenial::overflow(to, charged, level))?;
        }
        for (_, charge) in &mut charges {
            **charge = **charge - from + to;
        }
        Ok(())
    }

    fn lock_order(&self) -> Vec<(usize, &Arc<SerialLedger>)> {
        let mut order: Vec<_> = self.ledgers.iter().enumerate().collect();
        order.sort_unstable_by(|(_, a), (_, b)| a.issue_order.cmp(&b.issue_order));
        order
    }
}

#[cfg(test)]
mod lock_order_tests;
#[cfg(test)]
mod optional_tests;

impl ExecutionMemoryReservation {
    pub(in crate::authority) fn allocation_status(&self) -> Option<super::ExecutionLeaseStatus> {
        match &self.held {
            Held::Lease(memory) => Some(memory.status()),
            Held::Serial { .. } => None,
        }
    }

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
            .ok_or_else(|| LeaseDenial::refused(LeaseDenial::NoActiveExecutionScope))?
            .reserve(bytes)
            .map_err(LeaseDenial::MemoryExhausted)
    }

    pub(super) const fn lease(memory: LeaseMemory) -> Self {
        Self {
            held: Held::Lease(memory),
            activity: None,
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
        let activity = self.activity.take();
        let result = match (&mut self.held, lease) {
            (Held::Lease(memory), Some(lease)) => memory.rebind(lease, bytes),
            (
                Held::Serial {
                    budget,
                    bytes: held,
                },
                None,
            ) if serial.is_some_and(|active| budget.same_lineage(active)) => budget
                .resize(*held, bytes)
                .map_err(LeaseDenial::MemoryExhausted)
                .map(|()| {
                    *held = bytes;
                }),
            _ => Err(LeaseDenial::refused(LeaseDenial::UnrelatedNestedLease)),
        };
        if result.is_err() {
            self.activity = activity;
        }
        result
    }

    /// Holds `bytes` instead. A refused resize keeps what was held.
    pub fn resize(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let previous = self.bytes();
        self.resize_held(bytes)?;
        if let Some(activity) = &mut self.activity
            && let Err(denial) = activity.resize(bytes)
        {
            self.resize_held(previous)
                .expect("rolling back refused activity growth only releases memory");
            return Err(denial);
        }
        Ok(())
    }

    pub(crate) fn track_activity(&mut self, limits: &crate::backend::RunLimits) {
        debug_assert!(
            self.activity.is_none(),
            "report custody transfers at take-over"
        );
        self.activity = Some(limits.enter_memory(self.bytes()));
    }

    fn resize_held(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
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
