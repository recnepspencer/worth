use super::*;
impl InspectionAccess<'_> {
    pub(super) fn current_connectivity_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &ConnectivityInspectionRequest,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<ConnectivityInspectionSummary, crate::execution::RelationalExecutionDenialCause>
    {
        let partition_scope = PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
        let relation_kind_scope = KindScopeFilter::from_scope(request.relation_kind_scope.as_ref());
        let mut entities = Vec::new();
        let mut work = ConnectivityWork::default();

        for partition_id in self.current_partition_ids() {
            if !partition_scope.allows(partition_id) {
                continue;
            }
            let Some(partition) = self.current_partition_state(partition_id) else {
                continue;
            };
            for slot in partition.entity_arena.live_bitset.set_slots() {
                context
                    .checkpoint(1)
                    .map_err(crate::execution::denial::kernel_stop)?;
                let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                work.record_entity_scan();
                if work.work_units > request.budget.max_work_units {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    ));
                }
                if entities.len() as u64 >= request.budget.max_entities {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::EntityBudgetExceeded,
                    ));
                }
                entities.push(EntityId::new(
                    partition_id,
                    slot as u64,
                    slot_view.generation(),
                ));
            }
        }
        entities.sort();
        let entity_set = entities.iter().copied().collect::<BTreeSet<_>>();
        let mut adjacency = BTreeMap::<_, BTreeSet<_>>::new();
        let mut seen_relations = BTreeSet::<RelationId>::new();

        for entity in &entities {
            adjacency.entry(*entity).or_default();
            for relation_id in
                self.scoped_relation_ids_for_entity(execution, &InspectionScope::Current, *entity)?
            {
                context
                    .checkpoint(1)
                    .map_err(crate::execution::denial::kernel_stop)?;
                if !seen_relations.insert(relation_id) {
                    continue;
                }
                work.record_relation_scan();
                if work.work_units > request.budget.max_work_units {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    ));
                }
                if work.relation_scans > request.budget.max_relations {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::RelationBudgetExceeded,
                    ));
                }
                let Some(record) = self.scoped_authoritative_relation_record(
                    execution,
                    &InspectionScope::Current,
                    relation_id,
                )?
                else {
                    continue;
                };
                let kind_id = record.kind.kind_id;
                if !relation_kind_scope.allows(kind_id) {
                    continue;
                }
                if entity_set.contains(&record.source) && entity_set.contains(&record.target) {
                    adjacency
                        .entry(record.source)
                        .or_default()
                        .insert(record.target);
                    adjacency
                        .entry(record.target)
                        .or_default()
                        .insert(record.source);
                }
            }
        }

        let components = self.connectivity_components(
            request,
            self.runtime.current_version_id(),
            &entities,
            &adjacency,
            work,
            context,
        )?;
        Ok(match components {
            Ok((components, work)) => {
                let summary = ConnectivityInspectionSummary {
                    scope: request.scope.clone(),
                    version_id: self.runtime.current_version_id(),
                    component_count: components.len() as u64,
                    largest_component_size: components
                        .iter()
                        .map(|component| component.member_count)
                        .max()
                        .unwrap_or(0),
                    enumerated_entity_count: entities.len() as u64,
                    components,
                    origin: InspectionOrigin::CurrentTruth,
                    access_path: InspectionAccessPath::DirectLookup,
                    resolution_context: InspectionResolutionContext::ConnectivityTraversal,
                    availability: InspectionAvailability::Direct,
                    degradations: summary_degradations(request.include_members, None),
                };
                self.record_connectivity_work(&work);
                summary
            }
            Err((work, degradation)) => self.connectivity_budget_refusal(
                request,
                self.runtime.current_version_id(),
                work,
                degradation,
            ),
        })
    }
}
