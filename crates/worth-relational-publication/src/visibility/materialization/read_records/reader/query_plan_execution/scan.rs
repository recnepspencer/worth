use worth_relational_branch::internal_port as branch;
use branch::visibility::materialization::read_records::reader::query_plan_execution::scan_selection::*;
use super::*;

impl RelationalPinnedRead {
    pub(super) fn execute_scan_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        let basis = &self.basis;
        let kind = query_kind(&plan.packet.scope);
        // Kind candidates are authenticated root facts. Payload admission is
        // deferred to each reached Head in the streamed reader.
        if kind.is_none() {
            if let Some(partitions) = query_partition_scope(&plan.packet.scope) {
                basis
                    .demand_regions_full(execution, partitions.iter().copied())
                    .map_err(|cause| QueryReadExecutionStop::ReadRequest(cause.into()))?;
            } else {
                basis
                    .demand_regions_full(execution, basis.projection_root().partition_ids_iter())
                    .map_err(|cause| QueryReadExecutionStop::ReadRequest(cause.into()))?;
            }
        }
        let candidate_regions = kind.map_or_else(
            || basis.projection_root().region_count(),
            |(entity, kind)| kind_regions(basis, entity, kind).count(),
        );
        self.run_query_packets(plan, lease, execution, candidate_regions,
            |plan, budget| {
                let (entities, relations) =
                    if query_has_explicit_partition_scope(&plan.packet.scope) {
                        (Vec::new(), Vec::new())
                    } else if let Some((entity, kind)) = kind {
                        let partitions = leased_kind_partitions(basis, entity, kind, budget)?;
                        if entity {
                            (partitions, Vec::new())
                        } else {
                            (Vec::new(), partitions)
                        }
                    } else {
                        leased_scan_partitions(basis, budget)?
                    };
                prepare_scan_packets(&plan.packet, &entities, &relations, budget)
            },
            |plan, packets, remaining| {
                execute_leased_query_packets(
            packets,
            execution,
            remaining,
            |packet, ordinal, ceiling, context| {
                branch::visibility::materialization::read_records::reader::query_execution::leased_scan::execute_leased_scan_fragment(
                    execution,
                    self,
                    &plan.packet,
                    packet,
                    ordinal as u64,
                    ceiling,
                    context,
                )
            },
        )
            },
        )
    }
}
