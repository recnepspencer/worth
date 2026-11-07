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
                    .demand_regions_full(execution, basis.root().partition_ids_iter())
                    .map_err(|cause| QueryReadExecutionStop::ReadRequest(cause.into()))?;
            }
        }
        let candidate_regions = kind.map_or_else(
            || basis.root().region_count(),
            |(entity, kind)| kind_regions(basis.root(), entity, kind).count(),
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
                        let partitions =
                            leased_kind_partitions(basis.root(), entity, kind, budget)?;
                        if entity {
                            (partitions, Vec::new())
                        } else {
                            (Vec::new(), partitions)
                        }
                    } else {
                        leased_scan_partitions(basis.root(), basis, budget)?
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
    query_partition_scope(scope).is_some()
}
fn query_partition_scope(scope: &QueryScope) -> Option<&[crate::identity::data::PartitionId]> {
    match scope {
        QueryScope::EntityKindScan {
            partition_scope, ..
        }
        | QueryScope::RelationKindScan {
            partition_scope, ..
        }
        | QueryScope::EntityFieldEquals {
            partition_scope, ..
        }
        | QueryScope::RelationFieldEquals {
            partition_scope, ..
        }
        | QueryScope::EntityFieldAnyOf {
            partition_scope, ..
        }
        | QueryScope::RelationFieldAnyOf {
            partition_scope, ..
        }
        | QueryScope::AspectFilteredEntities {
            partition_scope, ..
        }
        | QueryScope::AspectFilteredRelations {
            partition_scope, ..
        } => partition_scope.as_deref(),
        _ => None,
    }
}

fn leased_scan_partitions(
    root: &crate::branch::RelationalBranchRoot,
    state: &dyn PartitionAccess,
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
            let Some(partition) = state.get_partition(partition_id) else {
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

fn query_kind(scope: &QueryScope) -> Option<(bool, crate::identity::data::KindId)> {
    match scope {
        QueryScope::EntityKindScan { kind_id, .. }
        | QueryScope::AspectFilteredEntities {
            kind_id: Some(kind_id),
            ..
        } => Some((true, *kind_id)),
        QueryScope::RelationKindScan { kind_id, .. }
        | QueryScope::AspectFilteredRelations {
            kind_id: Some(kind_id),
            ..
        } => Some((false, *kind_id)),
        _ => None,
    }
}
fn kind_regions(
    root: &crate::branch::RelationalBranchRoot,
    entity: bool,
    kind: crate::identity::data::KindId,
) -> Box<dyn Iterator<Item = crate::identity::data::PartitionId> + '_> {
    if entity {
        Box::new(root.entity_regions_of_kind(kind))
    } else {
        Box::new(root.relation_regions_of_kind(kind))
    }
}
fn leased_kind_partitions(
    root: &crate::branch::RelationalBranchRoot,
    entity: bool,
    kind: crate::identity::data::KindId,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<Vec<crate::identity::data::PartitionId>, worth_execution::MapKernelFailure<()>> {
    let count = kind_regions(root, entity, kind).count();
    budget.claim_items::<crate::identity::data::PartitionId>(count)?;
    let mut partitions = Vec::with_capacity(count);
    for id in kind_regions(root, entity, kind) {
        budget.checkpoint(1)?;
        partitions.push(id);
    }
    Ok(partitions)
}
