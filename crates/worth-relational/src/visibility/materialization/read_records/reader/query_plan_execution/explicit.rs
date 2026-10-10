use super::*;

impl VisibilityReadContext<'_> {
    pub(super) fn execute_explicit_query_plan(
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
        let (packets, preparation_report) = match &plan.packet.scope {
            QueryScope::ExplicitTargets { targets } => {
                if let Some(lease) = lease {
                    let (packets, report) = prepare_query_packets(lease, |budget| {
                        prepare_explicit_packets(targets, budget)
                    })?;
                    (packets, Some(report))
                } else {
                    (packetized_explicit_target_work(targets), None)
                }
            }
            _ => return Ok(None),
        };
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(self.runtime, &plan, &metrics);
        let Some(basis) = resolve_snapshot_basis(self.runtime, &plan.snapshot) else {
            return Ok(None);
        };
        let remaining_lease = match (lease, preparation_report) {
            (Some(lease), Some(report)) => Some(remaining_work_lease(lease, report)?),
            _ => None,
        };
        let Some((fragments, report)) = execute_explicit_query_fragments_from_exact_basis(
            self,
            &plan,
            packets,
            &basis,
            remaining_lease.as_ref(),
        )
        .map_err(|stop| match preparation_report {
            Some(report) => combine_query_stop(stop, report),
            None => stop,
        })?
        else {
            return Ok(None);
        };
        let report = match (preparation_report, report) {
            (Some(preparation), Some(execution)) => {
                Some(combine_query_reports(preparation, execution))
            }
            (_, report) => report,
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
