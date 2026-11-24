mod inspection_counters;
mod lineage_counters;
mod merge_counters;
mod observation_counters;
mod preparation_counters;
mod query_projection_counters;
mod replay_authority_basis_counters;
mod schema_continuity_counters;
#[cfg(test)]
mod test_complexity_observability;

use super::complexity_counter_sink::ComplexityCounterSink;
use crate::runtime::{RelationalRuntime, RuntimeInstrumentation};

pub(crate) use replay_authority_basis_counters::ReplayLineageAuthorityIndexedSource;

pub struct PerformanceAccess<'runtime> {
    counters: ComplexityCounterSink<'runtime>,
}

impl RelationalRuntime {
    pub(crate) fn performance_access(&self) -> PerformanceAccess<'_> {
        PerformanceAccess::new(self)
    }
}

impl<'runtime> PerformanceAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self::from_instrumentation(&runtime.services.instrumentation)
    }

    pub(crate) fn from_instrumentation(instrumentation: &'runtime RuntimeInstrumentation) -> Self {
        Self::from_counters(&instrumentation.complexity_counters)
    }

    pub(crate) fn from_counters(
        counters: &'runtime std::sync::Mutex<crate::performance::data::RuntimeComplexityCounters>,
    ) -> Self {
        Self {
            counters: ComplexityCounterSink::new(counters),
        }
    }

    pub(crate) fn complexity_counters_snapshot(
        &self,
    ) -> crate::performance::data::RuntimeComplexityCounters {
        self.counters.snapshot()
    }
}
