mod explicit;
mod scan;
mod traversal;
use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::query_execution::{
    execute_explicit_query_fragments_from_exact_basis, execute_leased_query_packets,
    query_execution_outcome_maybe_leased, record_query_packet_metrics, PacketizedQueryMetrics,
    QueryLeasedReadOutcome, QueryReadExecutionStop,
};
use super::query_preparation::{
    combine_query_reports, combine_query_stop, consult_query_preparation_request,
    prepare_query_packets, remaining_work_lease, QueryPreparationBudget,
};
use super::query_preparation_packetization::{
    prepare_explicit_packets, prepare_scan_packets, prepare_traversal_packets,
};
use super::*;

type QueryReadResult =
    Result<Option<(QueryExecutionOutcome, Option<ExecutionReport>)>, QueryReadExecutionStop>;

impl RelationalPinnedRead {
    /// Executes the pinned plan within the caller's request.
    pub fn execute_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<QueryLeasedReadOutcome>, QueryReadExecutionStop> {
        consult_query_preparation_request(execution)?;
        let read = execution
            .in_scope(|lease| self.execute_query_plan_internal(plan, lease, execution))
            .map_err(QueryReadExecutionStop::Request)?;
        read.map(|read| read.map(|(outcome, report)| QueryLeasedReadOutcome { outcome, report }))
    }

    fn count_query_backing(&self, lease: Option<&ExecutionResourceLease<'_>>) {
        if lease.is_some() {
            self.performance_access()
                .count_query_staged_parallel_strategy();
        } else {
            self.performance_access().count_query_serial_strategy();
        }
    }

    fn execute_query_plan_internal(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        if plan.snapshot != self.snapshot || plan.packet.context_id != self.context {
            return Ok(None);
        }
        if matches!(plan.packet.scope, QueryScope::ExplicitTargets { .. }) {
            return self.execute_explicit_query_plan(plan, lease, execution);
        }
        if matches!(
            plan.packet.scope,
            QueryScope::OutgoingNeighborhood { .. }
                | QueryScope::IncomingNeighborhood { .. }
                | QueryScope::ConnectivityTraversal { .. }
        ) {
            return self.execute_traversal_query_plan(plan, lease, execution);
        }

        self.execute_scan_query_plan(plan, lease, execution)
    }
}

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
