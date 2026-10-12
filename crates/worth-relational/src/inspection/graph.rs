use std::collections::{BTreeMap, BTreeSet};

use crate::inspection::data::{
    GraphInspectionRequest, GraphInspectionSummary, InspectionAccessPath, InspectionAvailability,
    InspectionDegradation, InspectionOrigin, InspectionRecordClass, InspectionScope,
    KindInspectionRequest, KindInspectionSummary,
};

use super::access::{
    summary_degradations, InspectionAccess, KindScopeFilter, PartitionScopeFilter,
};

impl<'runtime> InspectionAccess<'runtime> {
    pub fn graph_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &GraphInspectionRequest,
    ) -> Result<GraphInspectionSummary, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let read_view = match request.scope {
            InspectionScope::Current => None,
            _ => self.read_view_for_scope(execution, &request.scope)?,
        };
        Ok((|| {
            self.count_graph_summary_request();
            if matches!(request.scope, InspectionScope::Current) {
                return self.current_graph_summary(request);
            }
            let version_id = self.scope_version_id(&request.scope);
            let Some(read_view) = read_view else {
                return self.unavailable_graph_summary(request, version_id);
            };
            let partition_scope =
                PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
            let relation_kind_scope =
                KindScopeFilter::from_scope(request.relation_kind_scope.as_ref());
            let mut entity_kinds = BTreeMap::<_, u64>::new();
            let mut relation_kinds = BTreeMap::<_, u64>::new();
            let mut partition_ids = BTreeSet::new();
            let mut entity_count = 0_u64;
            let mut relation_count = 0_u64;
            let mut work_units = 0_u64;
            for record in read_view
                .entities()
                .iter()
                .filter(|record| partition_scope.allows(record.entity_id.partition_id))
            {
                work_units += 1;
                if work_units > request.budget.max_work_units {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        version_id,
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                entity_count += 1;
                if entity_count > request.budget.max_entities {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        version_id,
                        InspectionDegradation::EntityBudgetExceeded,
                    );
                }
                *entity_kinds.entry(record.kind.kind_id).or_default() += 1;
                partition_ids.insert(record.entity_id.partition_id);
            }
            for record in read_view
                .relations()
                .iter()
                .filter(|record| partition_scope.allows(record.relation_id.partition_id))
                .filter(|record| relation_kind_scope.allows(record.kind.kind_id))
            {
                work_units += 1;
                if work_units > request.budget.max_work_units {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        version_id,
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                relation_count += 1;
                if relation_count > request.budget.max_relations {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        version_id,
                        InspectionDegradation::RelationBudgetExceeded,
                    );
                }
                *relation_kinds.entry(record.kind.kind_id).or_default() += 1;
                partition_ids.insert(record.relation_id.partition_id);
            }

            GraphInspectionSummary {
                scope: request.scope.clone(),
                version_id,
                partition_count: partition_ids.len() as u64,
                entity_count,
                relation_count,
                entity_kinds: entity_kinds.into_iter().collect(),
                relation_kinds: relation_kinds.into_iter().collect(),
                origin: InspectionOrigin::VisibilitySnapshot,
                access_path: self.scope_access_path(&request.scope, version_id),
                availability: self.scope_availability(&request.scope, version_id),
                degradations: summary_degradations(!request.summary_only, None),
            }
        })())
    }

    pub fn kind_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &KindInspectionRequest,
    ) -> Result<KindInspectionSummary, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let read_view = self.read_view_for_scope(execution, &request.scope)?;
        Ok((|| {
            self.runtime
                .services
                .instrumentation
                .count(|counters| counters.inspection_kind_summary_requests += 1);
            let version_id = self.scope_version_id(&request.scope);
            let partition_scope =
                PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
            let Some(read_view) = read_view else {
                return KindInspectionSummary {
                    scope: request.scope.clone(),
                    version_id,
                    kind_id: request.kind_id,
                    record_class: request.record_class,
                    count: 0,
                    touched_partitions: Vec::new(),
                    origin: InspectionOrigin::VisibilitySnapshot,
                    access_path: self.scope_access_path(&request.scope, version_id),
                    availability: super::access::unavailable_scope_availability(&request.scope),
                };
            };
            let mut touched_partitions = BTreeSet::new();
            let count = match request.record_class {
                InspectionRecordClass::Entity => read_view
                    .entities()
                    .iter()
                    .filter(|record| partition_scope.allows(record.entity_id.partition_id))
                    .filter(|record| record.kind.kind_id == request.kind_id)
                    .map(|record| {
                        touched_partitions.insert(record.entity_id.partition_id);
                        1_u64
                    })
                    .sum(),
                InspectionRecordClass::Relation => read_view
                    .relations()
                    .iter()
                    .filter(|record| partition_scope.allows(record.relation_id.partition_id))
                    .filter(|record| record.kind.kind_id == request.kind_id)
                    .map(|record| {
                        touched_partitions.insert(record.relation_id.partition_id);
                        1_u64
                    })
                    .sum(),
            };
            KindInspectionSummary {
                scope: request.scope.clone(),
                version_id,
                kind_id: request.kind_id,
                record_class: request.record_class,
                count,
                touched_partitions: touched_partitions.into_iter().collect(),
                origin: InspectionOrigin::VisibilitySnapshot,
                access_path: self.scope_access_path(&request.scope, version_id),
                availability: self.scope_availability(&request.scope, version_id),
            }
        })())
    }

    fn current_graph_summary(&self, request: &GraphInspectionRequest) -> GraphInspectionSummary {
        let partition_scope = PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
        let relation_kind_scope = KindScopeFilter::from_scope(request.relation_kind_scope.as_ref());
        let mut entity_count = 0_u64;
        let mut relation_count = 0_u64;
        let mut entity_kinds = BTreeMap::<_, u64>::new();
        let mut relation_kinds = BTreeMap::<_, u64>::new();
        let mut touched_partitions = BTreeSet::new();
        let mut work_units = 0_u64;

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
                let Some(kind_id) = slot_view.kind_id() else {
                    continue;
                };
                work_units += 1;
                if work_units > request.budget.max_work_units {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        self.runtime.current_version_id(),
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                entity_count += 1;
                if entity_count > request.budget.max_entities {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        self.runtime.current_version_id(),
                        InspectionDegradation::EntityBudgetExceeded,
                    );
                }
                *entity_kinds.entry(kind_id).or_default() += 1;
                touched_partitions.insert(partition_id);
            }
            for slot in partition.relation_arena.live_bitset.iter_set_slots() {
                let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                let Some(kind_id) = slot_view.kind_id() else {
                    continue;
                };
                if !relation_kind_scope.allows(kind_id) {
                    continue;
                }
                work_units += 1;
                if work_units > request.budget.max_work_units {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        self.runtime.current_version_id(),
                        InspectionDegradation::WorkBudgetExceeded,
                    );
                }
                relation_count += 1;
                if relation_count > request.budget.max_relations {
                    self.count_budget_refusal();
                    return self.budget_exceeded_graph_summary(
                        request,
                        self.runtime.current_version_id(),
                        InspectionDegradation::RelationBudgetExceeded,
                    );
                }
                *relation_kinds.entry(kind_id).or_default() += 1;
                touched_partitions.insert(partition_id);
            }
        }

        GraphInspectionSummary {
            scope: request.scope.clone(),
            version_id: self.runtime.current_version_id(),
            partition_count: touched_partitions.len() as u64,
            entity_count,
            relation_count,
            entity_kinds: entity_kinds.into_iter().collect(),
            relation_kinds: relation_kinds.into_iter().collect(),
            origin: InspectionOrigin::CurrentTruth,
            access_path: InspectionAccessPath::DirectLookup,
            availability: InspectionAvailability::Direct,
            degradations: summary_degradations(!request.summary_only, None),
        }
    }
}
