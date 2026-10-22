use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

/// The caller's parallelism preference. Capability resolution may run either
/// posture serially, but may never weaken the determinism contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionPosture {
    Serial,
    Automatic,
}

/// Stable, data-derived identity of a partition. Worker position and pool
/// width never participate in this identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartitionIdentity(u64);

impl PartitionIdentity {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Stable identity of an installed equivalence predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EquivalenceContractId(u64);

impl EquivalenceContractId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeterminismContract {
    CanonicalBitwise,
    ContractEquivalent(EquivalenceContractId),
}

/// Required worker slots and optional charged-byte and operation ceilings.
/// Absence leaves accounting active without a caller ceiling. This grants no execution rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionBudget {
    max_workers: NonZeroUsize,
    charged_memory_bytes: Option<u64>,
    work_ceiling: Option<u64>,
}

impl ExecutionBudget {
    pub const fn new(
        max_workers: NonZeroUsize,
        charged_memory_bytes: Option<u64>,
        work_ceiling: Option<u64>,
    ) -> Self {
        Self {
            max_workers,
            charged_memory_bytes,
            work_ceiling,
        }
    }

    pub const fn max_workers(self) -> NonZeroUsize {
        self.max_workers
    }

    pub const fn charged_memory_bytes(self) -> Option<u64> {
        self.charged_memory_bytes
    }

    pub const fn work_ceiling(self) -> Option<u64> {
        self.work_ceiling
    }
}

/// Portable intent resolved before a runtime acquires a lease. Deadlines and
/// live cancellation belong to the runtime request that wraps this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionRequestPolicy {
    posture: ExecutionPosture,
    determinism: DeterminismContract,
    admission: ExecutionAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExecutionAdmission {
    WorkerOnly(NonZeroUsize),
    Accounted(ExecutionBudget),
}

impl ExecutionRequestPolicy {
    pub const fn new(
        posture: ExecutionPosture,
        determinism: DeterminismContract,
        budget: ExecutionBudget,
    ) -> Self {
        Self {
            posture,
            determinism,
            admission: ExecutionAdmission::Accounted(budget),
        }
    }

    pub const fn for_workers(
        posture: ExecutionPosture,
        determinism: DeterminismContract,
        max_workers: NonZeroUsize,
    ) -> Self {
        Self {
            posture,
            determinism,
            admission: ExecutionAdmission::WorkerOnly(max_workers),
        }
    }

    pub const fn posture(self) -> ExecutionPosture {
        self.posture
    }

    pub const fn determinism(self) -> DeterminismContract {
        self.determinism
    }

    pub const fn max_workers(self) -> NonZeroUsize {
        match self.admission {
            ExecutionAdmission::WorkerOnly(workers) => workers,
            ExecutionAdmission::Accounted(budget) => budget.max_workers(),
        }
    }

    pub const fn accounting(self) -> Option<ExecutionBudget> {
        match self.admission {
            ExecutionAdmission::WorkerOnly(_) => None,
            ExecutionAdmission::Accounted(budget) => Some(budget),
        }
    }

    pub const fn work_ceiling(self) -> Option<u64> {
        match self.accounting() {
            Some(budget) => budget.work_ceiling(),
            None => None,
        }
    }

    pub const fn charged_memory_bytes(self) -> Option<u64> {
        match self.accounting() {
            Some(budget) => budget.charged_memory_bytes(),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::{DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy};

    #[test]
    fn portable_request_preserves_independent_limits() {
        let budget = ExecutionBudget::new(NonZeroUsize::new(3).unwrap(), Some(4096), Some(77));
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            budget,
        );
        assert_eq!(policy.max_workers().get(), 3);
        assert_eq!(
            policy.accounting().unwrap().charged_memory_bytes(),
            Some(4096)
        );
        assert_eq!(policy.accounting().unwrap().work_ceiling(), Some(77));
        assert_eq!(policy.determinism(), DeterminismContract::CanonicalBitwise);
    }

    #[test]
    fn worker_only_intent_has_one_width_and_no_accounting_contract() {
        let policy = ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::new(16).unwrap(),
        );
        assert_eq!(policy.max_workers().get(), 16);
        assert_eq!(policy.accounting(), None);
        assert_eq!(policy.work_ceiling(), None);
        assert_eq!(policy.charged_memory_bytes(), None);
    }
}
