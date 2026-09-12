use super::basis::ExecutedIsolationBasis;
use crate::{ExecutedIsolationEvidenceDenial, PhysicalIsolationCounterSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedIsolationEvidence {
    basis: ExecutedIsolationBasis,
    counters: PhysicalIsolationCounterSnapshot,
}

impl ExecutedIsolationEvidence {
    #[cfg(any(test, feature = "certification-authority"))]
    pub fn from_foreground_reservation_test_counts(
        wait_count: u64,
        retry_count: u64,
    ) -> Result<Self, ExecutedIsolationEvidenceDenial> {
        let counters = PhysicalIsolationCounterSnapshot::from_store_executed_counts(
            1,
            wait_count,
            retry_count,
            1,
            1,
            1,
            1,
            1,
            4096,
        )?;
        let proof_progression_identity =
            super::project_counters::foreground_reservation_test_progression_identity(counters);
        let basis =
            ExecutedIsolationBasis::from_executed_isolation(proof_progression_identity, counters);
        Ok(Self { basis, counters })
    }

    pub const fn basis(&self) -> ExecutedIsolationBasis {
        self.basis
    }

    pub const fn counters(&self) -> PhysicalIsolationCounterSnapshot {
        self.counters
    }
}
