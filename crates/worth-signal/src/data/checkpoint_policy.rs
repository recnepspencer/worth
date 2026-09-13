//! Barrier policy for domain refresh scheduling.

use crate::data::checkpoint::CheckpointBarrier;

/// Per-domain barrier schedule.
#[derive(Debug, Clone)]
pub struct CheckpointPolicy<D: Copy + Ord> {
    default_barrier: CheckpointBarrier,
    per_domain: crate::data::persistent_ord_map::PersistentOrdMap<D, CheckpointBarrier>,
}

impl<D: Copy + Ord> CheckpointPolicy<D> {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            default_barrier: self.default_barrier,
            per_domain: self.per_domain.fork_persistent(),
        }
    }

    /// Create a new policy with one default barrier for all domains.
    pub fn new(default_barrier: CheckpointBarrier) -> Self {
        Self {
            default_barrier,
            per_domain: crate::data::persistent_ord_map::PersistentOrdMap::new(),
        }
    }

    /// Override barrier for one domain.
    pub fn set_barrier(&mut self, domain: D, barrier: CheckpointBarrier) {
        self.per_domain.insert(domain, barrier);
    }

    /// Resolve barrier for one domain.
    pub fn barrier_for(&self, domain: D) -> CheckpointBarrier {
        self.per_domain
            .get(&domain)
            .copied()
            .unwrap_or(self.default_barrier)
    }

    /// Return the fallback barrier used when no domain-specific override exists.
    pub fn barrier_for_default(&self) -> CheckpointBarrier {
        self.default_barrier
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            default_barrier: self.default_barrier,
            per_domain: self.per_domain.fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.per_domain.ptr_eq(&other.per_domain)
    }
}
