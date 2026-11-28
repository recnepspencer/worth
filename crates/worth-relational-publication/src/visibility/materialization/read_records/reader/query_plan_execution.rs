use worth_relational_branch::internal_port as branch;
pub mod explicit;
pub mod scan;
pub mod traversal;
use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::pinned_read::RelationalPinnedRead;
use branch::visibility::materialization::read_records::reader::query_execution::fragment_execution::execute_explicit_query_fragments_from_exact_basis;
use branch::visibility::materialization::read_records::reader::query_execution::leased_map::execute_leased_query_packets;
use branch::visibility::materialization::read_records::reader::query_execution::outcome::query_execution_outcome_maybe_leased;
use branch::visibility::materialization::read_records::reader::query_execution::packet_metrics::record_query_packet_metrics;
use branch::visibility::materialization::read_records::reader::query_execution::packet_metrics::PacketizedQueryMetrics;
use branch::visibility::materialization::read_records::reader::query_execution::leased_map::QueryLeasedReadOutcome;
use branch::visibility::materialization::read_records::reader::query_execution::leased_map::QueryReadExecutionStop;
use branch::visibility::materialization::read_records::reader::query_preparation::combine_query_reports;
use branch::visibility::materialization::read_records::reader::query_preparation::combine_query_stop;
use branch::visibility::materialization::read_records::reader::query_preparation::consult_query_preparation_request;
use branch::visibility::materialization::read_records::reader::query_preparation::prepare_query_packets;
use branch::visibility::materialization::read_records::reader::query_preparation::remaining_work_lease;
use branch::visibility::materialization::read_records::reader::query_preparation_packetization::prepare_explicit_packets;
use branch::visibility::materialization::read_records::reader::query_preparation_packetization::prepare_scan_packets;
use branch::visibility::materialization::read_records::reader::query_preparation_packetization::prepare_traversal_packets;
use worth_relational_contracts::internal_port::query::data::planned_packets::QueryScope;
use branch::query::data::query_outcome::QueryExecutionOutcome;
use branch::query::data::snapshot_pinned_plan::SnapshotPinnedQueryPlan;
use worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess;

use branch::query::data::QueryWorkerFragment;
use branch::visibility::materialization::read_records::reader::query_packetization::PacketizedQueryWork;
use branch::visibility::materialization::read_records::reader::query_preparation::QueryPreparationBudget;
use branch::visibility::materialization::read_records::reader::query_preparation_extent::declared_bytes;

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

    fn run_query_packets(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate_regions: usize,
        prepare: impl FnOnce(
            &SnapshotPinnedQueryPlan,
            &mut QueryPreparationBudget<'_, '_, '_>,
        ) -> Result<Vec<PacketizedQueryWork>, worth_execution::MapKernelFailure<()>>,
        execute: impl FnOnce(
            &SnapshotPinnedQueryPlan,
            Vec<PacketizedQueryWork>,
            Option<&ExecutionResourceLease<'_>>,
        ) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop>,
    ) -> QueryReadResult {
        let declared_bytes = declared_bytes(&plan.packet, candidate_regions)?;
        let (packets, preparation) =
            prepare_query_packets(execution, lease, declared_bytes, |budget| prepare(&plan, budget))?;
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(self.pinned_instrumentation(), &plan, &metrics);
        let remaining = lease
            .map(|lease| remaining_work_lease(lease, preparation))
            .transpose()?;
        if lease.is_some() {
            self.instrumentation.count_query_staged_parallel_strategy();
        } else {
            self.instrumentation.count_query_serial_strategy();
        }
        let (fragments, report) = execute(&plan, packets, remaining.as_ref())
            .map_err(|stop| combine_query_stop(stop, preparation))?;
        Ok(Some(query_execution_outcome_maybe_leased(
            self.pinned_instrumentation(),
            plan,
            metrics.packet_count,
            metrics.touched_partitions,
            metrics.target_count,
            fragments,
            execution,
            lease,
            Some(combine_query_reports(preparation, report)),
        )?))
    }

    pub(super) fn execute_query_plan_internal(
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

#[doc(hidden)]
pub trait RelationalPinnedReadOwnerPort: Sized {
    fn execute_query_plan_internal(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult;
}

impl RelationalPinnedReadOwnerPort for RelationalPinnedRead {
    fn execute_query_plan_internal(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        self.execute_query_plan_internal(plan, lease, execution)
    }
}
