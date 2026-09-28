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

pub(crate) use replay_authority_basis_counters::ReplayLineageAuthorityIndexedSource;

pub struct PerformanceAccess<'runtime> {
    pub(super) runtime: PerformanceOwnerView<'runtime>,
}

pub(super) struct PerformanceOwnerView<'runtime> {
    pub(super) services: PerformanceServicesView<'runtime>,
}

pub(super) struct PerformanceServicesView<'runtime> {
    pub(super) instrumentation: &'runtime RuntimeInstrumentation,
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
        Self {
            runtime: PerformanceOwnerView {
                services: PerformanceServicesView { instrumentation },
            },
        }
    }

    pub(crate) fn complexity_counters_snapshot(
        &self,
    ) -> crate::performance::data::RuntimeComplexityCounters {
        self.runtime
            .services
            .instrumentation
            .complexity_counters
            .lock()
            .expect("complexity counter lock poisoned")
            .clone()
    }
}
