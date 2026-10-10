use super::*;

impl VisibilityReadContext<'_> {
    pub(super) fn execute_traversal_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        if lease.is_none() {
            execution.check_live().map_err(|stop| {
                QueryReadExecutionStop::Request(worth_execution::WorkCeilingDenial::Stopped(stop))
            })?;
        }
        let (packets, preparation_report) = if let Some(lease) = lease {
            let (packets, report) = prepare_query_packets(lease, |budget| {
                prepare_traversal_packets(&plan.packet, budget)
            })?;
            (packets, Some(report))
        } else {
            let Some(packets) = packetized_traversal_query_work(&plan.packet) else {
                return Ok(None);
            };
            (packets, None)
        };
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(self.runtime, &plan, &metrics);
        let Some(basis) = resolve_snapshot_basis(self.runtime, &plan.snapshot) else {
            return Ok(None);
        };
        let state_access: &(dyn PartitionAccess + Sync) = basis.root().as_ref();
        let registry = basis.root().schema_authority().registry();
        let version_id = basis.version_id();
        let (fragments, report): QueryFragments = if let Some(lease) = lease {
            let preparation = preparation_report.expect("leased preparation has report");
            let remaining_lease = remaining_work_lease(lease, preparation)?;
            self.runtime
                .performance_access()
                .count_query_staged_parallel_strategy();
            let (fragments, report) = execute_leased_query_packets(
                packets,
                &remaining_lease,
                |packet, ordinal, ceiling, context| {
                    super::query_execution::execute_leased_traversal_fragment(
                        self.runtime,
                        &basis,
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
            (fragments, Some(combine_query_reports(preparation, report)))
        } else {
            self.runtime
                .performance_access()
                .count_query_serial_strategy();
            let mut scratch = QueryFragmentScratch::default();
            let Some(fragments) = packets
                .iter()
                .enumerate()
                .map(|(ordinal, packet)| {
                    execute_traversal_query_fragment_from_state(
                        self.runtime,
                        state_access,
                        registry,
                        version_id,
                        &plan.packet,
                        packet,
                        ordinal as u64,
                        &mut scratch,
                    )
                })
                .collect()
            else {
                return Ok(None);
            };
            (fragments, None)
        };
        Ok(Some(query_execution_outcome_maybe_leased(
            self.runtime,
            plan,
            metrics.packet_count,
            metrics.touched_partitions,
            metrics.target_count,
            fragments,
            lease,
            report,
        )?))
    }
}
