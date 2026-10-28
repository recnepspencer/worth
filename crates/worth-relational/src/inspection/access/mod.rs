mod history;
mod scope;
mod storage;
mod telemetry;

use crate::runtime::RelationalRuntime;

pub(crate) use scope::{
    empty_retention_plan, summary_degradations, unavailable_scope_availability, KindScopeFilter,
    PartitionScopeFilter,
};

pub struct InspectionAccess<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn inspection_access(&self) -> InspectionAccess<'_> {
        InspectionAccess::new(self)
    }
}

impl<'runtime> InspectionAccess<'runtime> {
    /// Operation demand: Full mirror custody before an inspection can read it.
    pub(super) fn demand_mirror(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::runtime::ResidentPartitionEdition,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.runtime
            .acquire_partition_edition()
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(Into::into)
    }

    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl InspectionAccess<'_> {
    /// Observe this runtime's actual reservations on the current thread. The
    /// explicit bound and allocation policy apply only to diagnostic evidence.
    pub fn observe_record_reservations(
        &self,
        maximum_records: usize,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::runtime::RecordReservationObservation,
        crate::runtime::RecordReservationObservationDenial,
    > {
        self.runtime
            .record_identity
            .observe_reservations(maximum_records, policy)
    }
}
