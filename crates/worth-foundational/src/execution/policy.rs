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

/// Limits the worker slots, charged bytes, and charged operation units of one
/// request. The runtime enforces these; this value grants no execution rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionBudget {
    max_workers: NonZeroUsize,
    charged_memory_bytes: u64,
    work_ceiling: u64,
}

impl ExecutionBudget {
    pub const fn new(
        max_workers: NonZeroUsize,
        charged_memory_bytes: u64,
        work_ceiling: u64,
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

    pub const fn charged_memory_bytes(self) -> u64 {
        self.charged_memory_bytes
    }

    pub const fn work_ceiling(self) -> u64 {
        self.work_ceiling
    }
}

/// Portable intent resolved before a runtime acquires a lease. Deadlines and
/// live cancellation belong to the runtime request that wraps this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExecutionRequestPolicy {
    posture: ExecutionPosture,
    determinism: DeterminismContract,
    budget: ExecutionBudget,
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
            budget,
        }
    }

    pub const fn posture(self) -> ExecutionPosture {
        self.posture
    }

    pub const fn determinism(self) -> DeterminismContract {
        self.determinism
    }

    pub const fn budget(self) -> ExecutionBudget {
        self.budget
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::{DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy};

    #[test]
    fn portable_request_preserves_independent_limits() {
        let budget = ExecutionBudget::new(NonZeroUsize::new(3).unwrap(), 4096, 77);
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            budget,
        );
        assert_eq!(policy.budget().max_workers().get(), 3);
        assert_eq!(policy.budget().charged_memory_bytes(), 4096);
        assert_eq!(policy.budget().work_ceiling(), 77);
        assert_eq!(policy.determinism(), DeterminismContract::CanonicalBitwise);
    }
}
