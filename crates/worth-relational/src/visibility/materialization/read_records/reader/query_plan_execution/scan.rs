use super::*;

impl RelationalPinnedRead {
    pub(super) fn execute_scan_query_plan(
        &self,
        plan: SnapshotPinnedQueryPlan,
        lease: Option<&ExecutionResourceLease<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> QueryReadResult {
        let basis = &self.basis;
        let declared_bytes = super::super::query_preparation_extent::declared_bytes(
            &plan.packet,
            basis.root().region_count(),
        )?;
        let (packets, preparation) =
            prepare_query_packets(execution, lease, declared_bytes, |budget| {
                let (entities, relations) =
                    if query_has_explicit_partition_scope(&plan.packet.scope) {
                        (Vec::new(), Vec::new())
                    } else {
                        leased_scan_partitions(basis.root(), budget)?
                    };
                prepare_scan_packets(&plan.packet, &entities, &relations, budget)
            })?;
        let metrics = PacketizedQueryMetrics::from_packets(&packets);
        record_query_packet_metrics(&self.performance_access(), &plan, &metrics);
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
