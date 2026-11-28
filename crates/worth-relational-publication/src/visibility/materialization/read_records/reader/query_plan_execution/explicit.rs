use super::*;

impl RelationalPinnedRead {
    pub(super) fn execute_explicit_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        if !matches!(plan.packet.scope, QueryScope::ExplicitTargets { .. }) {
            return Ok(None);
        }
        self.run_query_packets(
            plan,
            lease,
            execution,
            0,
            |plan, budget| {
                let QueryScope::ExplicitTargets { targets } = &plan.packet.scope else {
                    unreachable!("explicit query scope checked before packet preparation");
                };
                prepare_explicit_packets(targets, budget)
            },
            |plan, packets, remaining| {
                execute_explicit_query_fragments_from_exact_basis(
                    self,
                    self.pinned_current_version(),
                    plan,
                    packets,
                    &self.basis,
                    execution,
                    remaining,
                )
            },
        )
    }
}
