use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::query_execution::{
    execute_explicit_query_fragments_from_exact_basis, execute_leased_query_packets,
    query_execution_outcome_maybe_leased, record_query_packet_metrics, PacketizedQueryMetrics,
    QueryLeasedReadOutcome, QueryReadExecutionStop,
};
use super::query_fragment_work::{
    execute_query_fragment, execute_traversal_query_fragment_from_state,
};
use super::query_packetization::{
    packetized_explicit_target_work, packetized_query_work, packetized_traversal_query_work,
};
use super::query_preparation::{
    combine_query_reports, combine_query_stop, prepare_query_packets, remaining_work_lease,
    QueryPreparationBudget,
};
use super::query_preparation_packetization::{
    prepare_explicit_packets, prepare_scan_packets, prepare_traversal_packets,
};
use super::*;

type QueryFragments = (
    Vec<crate::query::data::QueryWorkerFragment>,
    Option<ExecutionReport>,
);
type QueryReadResult =
    Result<Option<(QueryExecutionOutcome, Option<ExecutionReport>)>, QueryReadExecutionStop>;

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Reads on the calling thread without an execution lease.
    pub fn execute_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
    ) -> Option<QueryExecutionOutcome> {
        self.execute_query_plan_internal(plan, None)
            .expect("unleased reads cannot dispatch or stop")
            .map(|(outcome, _)| outcome)
    }

    pub fn execute_query_plan_with_lease(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<Option<QueryExecutionOutcome>, QueryReadExecutionStop> {
        self.execute_query_plan_with_lease_report(plan, lease)
            .map(|read| read.map(|read| read.outcome))
    }

    pub fn execute_query_plan_with_lease_report(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<Option<QueryLeasedReadOutcome>, QueryReadExecutionStop> {
        self.execute_query_plan_internal(plan, Some(lease))
            .map(|read| {
                read.map(|(outcome, report)| QueryLeasedReadOutcome {
                    outcome,
                    report: report.expect("leased query execution has a report"),
                })
            })
    }

    fn execute_query_plan_internal(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> QueryReadResult {
        let Some(context_id) = self.query_plan_context(&plan.snapshot) else {
            return Ok(None);
        };
        if plan.packet.context_id != context_id {
            return Ok(None);
        }
        if matches!(plan.packet.scope, QueryScope::ExplicitTargets { .. }) {
            return self.execute_explicit_query_plan(plan, lease);
        }
        if matches!(
            plan.packet.scope,
            QueryScope::OutgoingNeighborhood { .. }
                | QueryScope::IncomingNeighborhood { .. }
                | QueryScope::ConnectivityTraversal { .. }
        ) {
            return self.execute_traversal_query_plan(plan, lease);
        }

        let (read_view, basis, packets, preparation_report) = if let Some(lease) = lease {
            let Some(basis) = resolve_snapshot_basis(self.runtime, &plan.snapshot) else {
                return Ok(None);
            };
            let (packets, report) = prepare_query_packets(lease, |budget| {
                let (entities, relations) =
                    if query_has_explicit_partition_scope(&plan.packet.scope) {
                        (Vec::new(), Vec::new())
                    } else {
                        leased_scan_partitions(basis.root(), budget)?
                    };
                prepare_scan_packets(&plan.packet, &entities, &relations, budget)
            })?;
            (None, Some(basis), packets, Some(report))
        } else {
            let Some(read_view) = self.read_snapshot(&plan.snapshot) else {
                return Ok(None);
            };
            let entity_partitions = read_view
                .entities()
                .iter()
                .map(|record| record.entity_id.partition_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let relation_partitions = read_view
                .relations()
                .iter()
                .map(|record| record.relation_id.partition_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let Some(packets) =
                packetized_query_work(&plan.packet, &entity_partitions, &relation_partitions)
            else {
                return Ok(None);
            };
            (Some(read_view), None, packets, None)
        };
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(self.runtime, &plan, &metrics);
        let (fragments, report) = if let Some(lease) = lease {
            let preparation = preparation_report.expect("leased preparation has report");
            let remaining_lease = remaining_work_lease(lease, preparation)?;
            self.runtime
                .performance_access()
                .count_query_staged_parallel_strategy();
            let (fragments, report) = execute_leased_query_packets(
                packets,
                &remaining_lease,
                |packet, ordinal, ceiling, context| {
                    super::query_execution::execute_leased_scan_fragment(
                        self.runtime,
                        basis.as_ref().expect("leased scan has basis"),
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
            let read_view = read_view.as_ref().expect("serial scan has read view");
            let mut scratch = QueryFragmentScratch::default();
            let Some(fragments) = packets
                .iter()
                .enumerate()
                .map(|(ordinal, packet)| {
                    execute_query_fragment(
                        read_view,
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

    fn execute_explicit_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> QueryReadResult {
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

    fn execute_traversal_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> QueryReadResult {
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

fn query_has_explicit_partition_scope(scope: &QueryScope) -> bool {
    matches!(
        scope,
        QueryScope::EntityKindScan {
            partition_scope: Some(_),
            ..
        } | QueryScope::RelationKindScan {
            partition_scope: Some(_),
            ..
        } | QueryScope::EntityFieldEquals {
            partition_scope: Some(_),
            ..
        } | QueryScope::RelationFieldEquals {
            partition_scope: Some(_),
            ..
        } | QueryScope::EntityFieldAnyOf {
            partition_scope: Some(_),
            ..
        } | QueryScope::RelationFieldAnyOf {
            partition_scope: Some(_),
            ..
        } | QueryScope::AspectFilteredEntities {
            partition_scope: Some(_),
            ..
        } | QueryScope::AspectFilteredRelations {
            partition_scope: Some(_),
            ..
        }
    )
}

fn leased_scan_partitions(
    root: &crate::branch::RelationalBranchRoot,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<
    (
        Vec<crate::identity::data::PartitionId>,
        Vec<crate::identity::data::PartitionId>,
    ),
    worth_execution::MapKernelFailure<()>,
> {
    let region_count = root.region_count();
    budget.claim_items::<crate::identity::data::PartitionId>(region_count.saturating_mul(2))?;
    let mut entities = Vec::with_capacity(region_count);
    let mut relations = Vec::with_capacity(region_count);
    root.try_for_each_partition_id(
        |partition_id| -> Result<(), worth_execution::MapKernelFailure<()>> {
            budget.checkpoint(1)?;
            let Some(partition) = root.get_partition(partition_id) else {
                return Ok(());
            };
            let entity_present = partition.entity_arena.live_bitset.count_ones() > 0;
            let relation_present = partition.relation_arena.live_bitset.count_ones() > 0;
            if entity_present {
                entities.push(partition_id);
            }
            if relation_present {
                relations.push(partition_id);
            }
            Ok(())
        },
    )?;
    Ok((entities, relations))
}
