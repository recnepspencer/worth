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
            .demand_all(execution)
            .map_err(Into::into)
    }

    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
