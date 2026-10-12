use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::inspection::data::{
    ConnectivityComponentSummary, ConnectivityInspectionRequest, ConnectivityInspectionSummary,
    InspectionAccessPath, InspectionAvailability, InspectionDegradation, InspectionOrigin,
    InspectionResolutionContext, InspectionScope, NeighborInspectionResult,
};

use super::access::{
    summary_degradations, InspectionAccess, KindScopeFilter, PartitionScopeFilter,
};

#[derive(Default)]
struct ConnectivityWork {
    entity_scans: u64,
    relation_scans: u64,
    frontier_expansions: u64,
    components_evaluated: u64,
    work_units: u64,
}

impl ConnectivityWork {
    fn record_entity_scan(&mut self) {
        self.entity_scans += 1;
        self.work_units += 1;
    }

    fn record_relation_scan(&mut self) {
        self.relation_scans += 1;
        self.work_units += 1;
    }

    fn record_frontier_expansion(&mut self) {
        self.frontier_expansions += 1;
        self.work_units += 1;
    }

    fn record_component(&mut self) {
        self.components_evaluated += 1;
        self.work_units += 1;
    }
}

impl<'runtime> InspectionAccess<'runtime> {
    pub fn connectivity_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &ConnectivityInspectionRequest,
    ) -> ConnectivityInspectionSummary {
        self.count_connectivity_summary_request();
        if matches!(request.scope, InspectionScope::Current) {
            return self.current_connectivity_summary(execution, request);
        }
        let version_id = self.scope_version_id(&request.scope);
        let Some(read_view) = self.read_view_for_scope(execution, &request.scope) else {
            return self.unavailable_connectivity_summary(request, version_id);
        };
        let partition_scope = PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
        let relation_kind_scope = KindScopeFilter::from_scope(request.relation_kind_scope.as_ref());
        let mut work = ConnectivityWork::default();
        let mut entities = Vec::new();
        for entity_id in read_view
            .entities()
            .iter()
            .filter(|record| partition_scope.allows(record.entity_id.partition_id))
            .map(|record| record.entity_id)
        {
            work.record_entity_scan();
            if work.work_units > request.budget.max_work_units {
                return self.connectivity_budget_refusal(
                    request,
                    version_id,
                    work,
                    InspectionDegradation::WorkBudgetExceeded,
                );
            }
            if entities.len() as u64 >= request.budget.max_entities {
                return self.connectivity_budget_refusal(
                    request,
                    version_id,
                    work,
                    InspectionDegradation::EntityBudgetExceeded,
                );
            }
            entities.push(entity_id);
        }
        let entity_set = entities.iter().copied().collect::<BTreeSet<_>>();

        let mut adjacency = BTreeMap::<_, BTreeSet<_>>::new();
        for entity in &entities {
            adjacency.entry(*entity).or_default();
        }
        for relation in read_view
            .relations()
            .iter()
            .filter(|record| {
                entity_set.contains(&record.source) && entity_set.contains(&record.target)
            })
            .filter(|record| relation_kind_scope.allows(record.kind.kind_id))
        {
            work.record_relation_scan();
            if work.work_units > request.budget.max_work_units {
                return self.connectivity_budget_refusal(
                    request,
                    version_id,
                    work,
                    InspectionDegradation::WorkBudgetExceeded,
                );
            }
            if work.relation_scans > request.budget.max_relations {
                return self.connectivity_budget_refusal(
                    request,
                    version_id,
                    work,
                    InspectionDegradation::RelationBudgetExceeded,
                );
            }
            adjacency
                .entry(relation.source)
                .or_default()
                .insert(relation.target);
            adjacency
                .entry(relation.target)
                .or_default()
                .insert(relation.source);
        }

        let components =
            self.connectivity_components(request, version_id, &entities, &adjacency, work);
        match components {
            Ok((components, work)) => {
                let summary = ConnectivityInspectionSummary {
                    scope: request.scope.clone(),
                    version_id,
                    component_count: components.len() as u64,
                    largest_component_size: components
                        .iter()
                        .map(|component| component.member_count)
                        .max()
                        .unwrap_or(0),
                    enumerated_entity_count: entities.len() as u64,
                    components,
                    origin: InspectionOrigin::VisibilitySnapshot,
                    access_path: self.scope_access_path(&request.scope, version_id),
                    resolution_context: InspectionResolutionContext::ConnectivityTraversal,
                    availability: self.scope_availability(&request.scope, version_id),
                    degradations: summary_degradations(request.include_members, None),
                };
                self.record_connectivity_work(&work);
                summary
            }
            Err((work, degradation)) => {
                self.connectivity_budget_refusal(request, version_id, work, degradation)
            }
        }
    }

    pub fn neighbors(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: InspectionScope,
        entity_id: EntityId,
    ) -> NeighborInspectionResult {
        self.count_neighbor_request();
        let version_id = self.scope_version_id(&scope);
        let relation_ids = self.scoped_relation_ids_for_entity(execution, &scope, entity_id);
        let (outgoing_relation_ids, incoming_relation_ids): (Vec<_>, Vec<_>) =
            relation_ids.into_iter().partition(|relation_id| {
                self.scoped_relation_endpoints(execution, &scope, *relation_id)
                    .is_some_and(|(source, _)| source == entity_id)
            });
        NeighborInspectionResult {
            entity_id,
            version_id,
            outgoing_relation_ids,
            incoming_relation_ids,
            origin: InspectionOrigin::VisibilitySnapshot,
            access_path: self.scope_access_path(&scope, version_id),
            resolution_context: InspectionResolutionContext::RelationNeighborhood,
            availability: self.scope_availability(&scope, version_id),
        }
    }

    fn current_connectivity_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &ConnectivityInspectionRequest,
    ) -> ConnectivityInspectionSummary {
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
            for slot in partition.entity_arena.live_bitset.iter_set_slots() {
                let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                work.record_entity_scan();
                if work.work_units > request.budget.max_work_units {
                    return self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                if entities.len() as u64 >= request.budget.max_entities {
                    return self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::EntityBudgetExceeded,
                    );
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
                self.scoped_relation_ids_for_entity(execution, &InspectionScope::Current, *entity)
            {
                if !seen_relations.insert(relation_id) {
                    continue;
                }
                work.record_relation_scan();
                if work.work_units > request.budget.max_work_units {
                    return self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                if work.relation_scans > request.budget.max_relations {
                    return self.connectivity_budget_refusal(
                        request,
                        self.runtime.current_version_id(),
                        work,
                        InspectionDegradation::RelationBudgetExceeded,
                    );
                }
                let Some(record) = self.scoped_authoritative_relation_record(
                    execution,
                    &InspectionScope::Current,
                    relation_id,
                ) else {
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
        );
        match components {
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
        }
    }

    fn connectivity_components(
        &self,
        request: &ConnectivityInspectionRequest,
        _version_id: VersionId,
        entities: &[EntityId],
        adjacency: &BTreeMap<EntityId, BTreeSet<EntityId>>,
        mut work: ConnectivityWork,
    ) -> Result<
        (Vec<ConnectivityComponentSummary>, ConnectivityWork),
        (ConnectivityWork, InspectionDegradation),
    > {
        let mut visited = BTreeSet::new();
        let mut components = Vec::new();
        for entity in entities {
            if !visited.insert(*entity) {
                continue;
            }
            work.record_component();
            if work.work_units > request.budget.max_work_units {
                return Err((work, InspectionDegradation::WorkBudgetExceeded));
            }
            if work.components_evaluated > request.budget.max_components {
                return Err((work, InspectionDegradation::ComponentBudgetExceeded));
            }
            let mut queue = VecDeque::from([*entity]);
            let mut members = vec![*entity];
            while let Some(current) = queue.pop_front() {
                work.record_frontier_expansion();
                if work.work_units > request.budget.max_work_units {
                    return Err((work, InspectionDegradation::WorkBudgetExceeded));
                }
                if queue.len() as u64 > request.budget.max_frontier {
                    return Err((work, InspectionDegradation::FrontierBudgetExceeded));
                }
                for neighbor in adjacency.get(&current).into_iter().flatten() {
                    if visited.insert(*neighbor) {
                        members.push(*neighbor);
                        queue.push_back(*neighbor);
                    }
                }
            }
            members.sort();
            components.push(ConnectivityComponentSummary {
                member_count: members.len() as u64,
                members: request.include_members.then_some(members),
            });
        }
        Ok((components, work))
    }

    fn connectivity_budget_refusal(
        &self,
        request: &ConnectivityInspectionRequest,
        version_id: VersionId,
        work: ConnectivityWork,
        degradation: InspectionDegradation,
    ) -> ConnectivityInspectionSummary {
        self.record_connectivity_work(&work);
        self.count_budget_refusal();
        self.budget_exceeded_connectivity_summary(request, version_id, degradation)
    }

    fn record_connectivity_work(&self, work: &ConnectivityWork) {
        self.count_connectivity_work(
            work.entity_scans,
            work.relation_scans,
            work.frontier_expansions,
            work.components_evaluated,
        );
    }
}
