use super::PerformanceAccess;
use crate::performance::data::{
    ComplexityContract, RuntimeComplexityCounters, COMPLEXITY_CONTRACTS,
};

impl PerformanceAccess<'_> {
    pub fn contracts(&self) -> &'static [ComplexityContract] {
        COMPLEXITY_CONTRACTS
    }

    pub fn counters(&self) -> RuntimeComplexityCounters {
        self.complexity_counters_snapshot()
    }

    pub fn reset_counters(&self) {
        self.counters
            .count(|counts| *counts = RuntimeComplexityCounters::default());
    }
}
