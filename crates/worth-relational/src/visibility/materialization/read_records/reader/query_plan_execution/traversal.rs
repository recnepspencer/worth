use super::*;

impl RelationalPinnedRead {
    pub(super) fn execute_traversal_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        let declared_bytes =
            super::super::query_preparation_extent::declared_bytes(&plan.packet, 0)?;
        let (packets, preparation) =
            prepare_query_packets(execution, lease, declared_bytes, |budget| {
                prepare_traversal_packets(&plan.packet, budget)
            })?;
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(&self.performance_access(), &plan, &metrics);
        let basis = &self.basis;
        let state_access: &(dyn PartitionAccess + Sync) = basis.root().as_ref();
        let registry = basis.root().schema_authority().registry();
        let version_id = basis.version_id();
        let remaining_lease = lease
            .map(|lease| remaining_work_lease(lease, preparation))
            .transpose()?;
        self.count_query_backing(lease);
        let (fragments, report) = execute_leased_query_packets(
            packets,
            execution,
            remaining_lease.as_ref(),
            |packet, ordinal, ceiling, context| {
                super::query_execution::execute_leased_traversal_fragment(
                    self,
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
        .map_err(|stop| combine_query_stop(stop, preparation))?;
        let report = Some(combine_query_reports(preparation, report));
        Ok(Some(query_execution_outcome_maybe_leased(
            &self.performance_access(),
            plan,
            metrics.packet_count,
            metrics.touched_partitions,
            metrics.target_count,
            fragments,
            execution,
            lease,
            report,
        )?))
    }
}
