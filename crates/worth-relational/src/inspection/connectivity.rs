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

mod components;
mod current;
mod work;
use work::ConnectivityWork;

impl<'runtime> InspectionAccess<'runtime> {
    pub fn connectivity_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &ConnectivityInspectionRequest,
    ) -> Result<ConnectivityInspectionSummary, crate::execution::RelationalExecutionDenialCause>
    {
        if matches!(request.scope, InspectionScope::Current) {
            let resident = self.demand_mirror(execution)?;
            return crate::visibility::materialization::read_records::read_work::metered(
                execution,
                |context| self.current_connectivity_summary(&resident, execution, request, context),
                |_| 0,
            )?;
        }
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| self.connectivity_summary_funded(execution, request, context),
            |_| 0,
        )?
    }

    fn connectivity_summary_funded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &ConnectivityInspectionRequest,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<ConnectivityInspectionSummary, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        let read_view = match request.scope {
            InspectionScope::Current => None,
            _ => self.read_view_for_scope(execution, &request.scope)?,
        };
        (|| {
            self.count_connectivity_summary_request();
            let version_id = self.scope_version_id(&request.scope);
            let Some(read_view) = read_view else {
                return Ok(self.unavailable_connectivity_summary(request, version_id));
            };
            let partition_scope =
                PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
            let relation_kind_scope =
                KindScopeFilter::from_scope(request.relation_kind_scope.as_ref());
            let mut work = ConnectivityWork::default();
            let mut entities = Vec::new();
            for entity_id in read_view
                .entities()
                .iter()
                .filter(|record| partition_scope.allows(record.entity_id.partition_id))
                .map(|record| record.entity_id)
            {
                context
                    .checkpoint(1)
                    .map_err(crate::execution::denial::kernel_stop)?;
                work.record_entity_scan();
                if work.work_units > request.budget.max_work_units {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        version_id,
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    ));
                }
                if entities.len() as u64 >= request.budget.max_entities {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        version_id,
                        work,
                        InspectionDegradation::EntityBudgetExceeded,
                    ));
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
                context
                    .checkpoint(1)
                    .map_err(crate::execution::denial::kernel_stop)?;
                work.record_relation_scan();
                if work.work_units > request.budget.max_work_units {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        version_id,
                        work,
                        InspectionDegradation::WorkBudgetExceeded,
                    ));
                }
                if work.relation_scans > request.budget.max_relations {
                    return Ok(self.connectivity_budget_refusal(
                        request,
                        version_id,
                        work,
                        InspectionDegradation::RelationBudgetExceeded,
                    ));
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

            let components = self.connectivity_components(
                request, version_id, &entities, &adjacency, work, context,
            )?;
            Ok(match components {
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
            })
        })()
    }

    pub fn neighbors(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: InspectionScope,
        entity_id: EntityId,
    ) -> Result<NeighborInspectionResult, crate::execution::RelationalExecutionDenialCause> {
        let resident = self.demand_mirror(execution)?;
        self.count_neighbor_request();
        let version_id = self.scope_version_id(&scope);
        let relation_ids =
            self.scoped_relation_ids_for_entity(&resident, execution, &scope, entity_id)?;
        let mut outgoing_relation_ids = Vec::new();
        let mut incoming_relation_ids = Vec::new();
        for relation_id in relation_ids {
            if self
                .scoped_relation_endpoints(&resident, execution, &scope, relation_id)?
                .is_some_and(|(source, _)| source == entity_id)
            {
                outgoing_relation_ids.push(relation_id);
            } else {
                incoming_relation_ids.push(relation_id);
            }
        }
        Ok(NeighborInspectionResult {
            entity_id,
            version_id,
            outgoing_relation_ids,
            incoming_relation_ids,
            origin: InspectionOrigin::VisibilitySnapshot,
            access_path: self.scope_access_path(&scope, version_id),
            resolution_context: InspectionResolutionContext::RelationNeighborhood,
            availability: self.scope_availability(&scope, version_id),
        })
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
