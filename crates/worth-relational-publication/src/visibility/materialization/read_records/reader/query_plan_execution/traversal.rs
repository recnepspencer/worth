use super::*;

impl RelationalPinnedRead {
    pub(super) fn execute_traversal_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        self.run_query_packets(plan, lease, execution, 0,
            |plan, budget| prepare_traversal_packets(&plan.packet, budget),
            |plan, packets, remaining| {
        let basis = &self.basis;
        let state_access: &(dyn PartitionAccess + Sync) = basis;
        let registry = basis.projection_root().schema_authority().registry();
        let version_id = basis.version_id();
execute_leased_query_packets(
            packets,
            execution,
            remaining,
            |packet, ordinal, ceiling, context| {
                worth_relational_branch::internal_port::visibility::materialization::read_records::reader::query_execution::leased_traversal::execute_leased_traversal_fragment(
                    execution,
                    self,
                    self.pinned_current_version(),
                    state_access,
                    registry,
                    version_id,
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
