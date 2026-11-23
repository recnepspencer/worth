use super::context::VisibilityReadContext;
use super::pin_denial::RelationalPinnedReadAdmissionDenial;
use super::query_execution::{QueryLeasedReadOutcome, QueryReadExecutionStop};
use super::query_preparation::consult_query_preparation_request;
use crate::query::data::snapshot_pinned_plan::SnapshotPinnedQueryPlan;

impl VisibilityReadContext<'_> {
    /// Pin exact native custody before executing the existing read algorithm.
    pub fn execute_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<QueryLeasedReadOutcome>, QueryReadExecutionStop> {
        consult_query_preparation_request(execution)?;
        let read = execution
            .in_scope(|lease| {
                let read = match self.pin_read_capability(&plan.snapshot, &[], execution) {
                    Ok(read) => read,
                    Err(RelationalPinnedReadAdmissionDenial::Request(stop)) => {
                        return Err(super::query_preparation::query_preparation_read_stop(
                            stop, execution,
                        ));
                    }
                    Err(
                        RelationalPinnedReadAdmissionDenial::SnapshotUnavailable
                        | RelationalPinnedReadAdmissionDenial::IndexNotInstalled(_),
                    ) => return Ok(None),
                };
                read.execute_query_plan_internal(plan, lease, execution)
            })
            .map_err(QueryReadExecutionStop::Request)?;
        read.map(|read| read.map(|(outcome, report)| QueryLeasedReadOutcome { outcome, report }))
    }
}
