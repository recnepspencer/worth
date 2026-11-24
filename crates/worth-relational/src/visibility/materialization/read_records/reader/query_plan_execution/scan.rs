use super::scan_selection::*;
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
        let declared_bytes = super::super::query_preparation_extent::declared_bytes(
            &plan.packet,
            candidate_regions,
        )?;
        let (packets, preparation) =
            prepare_query_packets(execution, lease, declared_bytes, |budget| {
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
            })?;
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(self.pinned_instrumentation(), &plan, &metrics);
        let remaining = lease
            .map(|lease| remaining_work_lease(lease, preparation))
            .transpose()?;
        self.count_query_backing(lease);
        let (fragments, report) = execute_leased_query_packets(
            packets,
            execution,
            remaining.as_ref(),
            |packet, ordinal, ceiling, context| {
                super::super::query_execution::execute_leased_scan_fragment(
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
        .map_err(|stop| combine_query_stop(stop, preparation))?;
        let report = Some(combine_query_reports(preparation, report));
        Ok(Some(query_execution_outcome_maybe_leased(
            self.pinned_instrumentation(),
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
