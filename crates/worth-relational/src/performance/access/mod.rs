mod counter_sink;
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
mod validation_counters;
mod working_state_counters;

use crate::runtime::{RelationalRuntime, RuntimeInstrumentation};
use counter_sink::ComplexityCounterSink;

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

    pub(crate) fn accumulate_round_counts(
        &self,
        counts: crate::performance::data::RuntimeComplexityCounters,
    ) {
        self.counters.count(|target| {
            crate::performance::operation_complexity_accounting::accumulate_complexity_counts(
                target, counts,
            );
        });
    }

    pub(crate) fn complexity_counters_snapshot(
        &self,
    ) -> crate::performance::data::RuntimeComplexityCounters {
        self.counters.snapshot()
    }
}
