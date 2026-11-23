use std::collections::BTreeSet;

use crate::history::data::{
    AspectHistoryEntry, AspectHistoryQueryResult, BranchId, HistoryAspectQueryTarget,
    LineageAspectHistory, LineageAspectHistoryQueryResult,
};
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::HistoricalResolutionRequest;
use crate::transactions::data::RecordRef;
use crate::visibility::materialization::read_records::ProjectionAspectFilter;

use super::branch_envelope_reads::ASPECT_HISTORY_COMMIT_CEILING;
use super::HistoryAccess;
use crate::execution::RelationalExecutionDenialCause;

impl<'runtime> HistoryAccess<'runtime> {
    pub fn entity_aspect_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        entity_id: EntityId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<Vec<AspectHistoryEntry>, RelationalExecutionDenialCause> {
        Ok(self
            .entity_aspect_history_with_trace(execution, branch_id, entity_id, filter)?
            .entries)
    }

    pub fn relation_aspect_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        relation_id: RelationId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<Vec<AspectHistoryEntry>, RelationalExecutionDenialCause> {
        Ok(self
            .relation_aspect_history_with_trace(execution, branch_id, relation_id, filter)?
            .entries)
    }

    pub fn entity_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        entity_id: EntityId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<AspectHistoryQueryResult, RelationalExecutionDenialCause> {
        self.record_aspect_history_with_trace(
            execution,
            branch_id,
            RecordRef::Entity(entity_id),
            HistoryAspectQueryTarget::Entity(entity_id),
            filter,
        )
    }

    pub fn relation_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        relation_id: RelationId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<AspectHistoryQueryResult, RelationalExecutionDenialCause> {
        self.record_aspect_history_with_trace(
            execution,
            branch_id,
            RecordRef::Relation(relation_id),
            HistoryAspectQueryTarget::Relation(relation_id),
            filter,
        )
    }

    pub fn lineage_entity_aspect_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        lineage_id: LineageId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<Option<LineageAspectHistory>, crate::execution::RelationalExecutionDenialCause>
    {
        Ok(self
            .lineage_entity_aspect_history_with_trace(execution, branch_id, lineage_id, filter)?
            .history)
    }

    pub fn lineage_entity_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        lineage_id: LineageId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<LineageAspectHistoryQueryResult, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        let resolution = self
            .runtime
            .lineage_access()
            .resolve_historical_lineage(execution, HistoricalResolutionRequest {
                branch_id: branch_id.clone(),
                lineage_id,
                boundedness_basis:
                    crate::facade::lineage::HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            })?;
        let lineage_scope = self.lineage_scope(&resolution.start, &resolution.traversed_event_ids);
        let mut entity_scope = BTreeSet::new();
        for id in &lineage_scope {
            if let Some(node) = self.runtime.lineage.demand_node(execution, *id)? {
                entity_scope.insert(node.node().entity_id());
            }
        }
        self.demand_branch_history_regions(
            execution,
            branch_id,
            entity_scope.iter().map(|entity| entity.partition_id),
        )?;
        let envelopes = self.demand_branch_commit_envelopes(
            execution,
            branch_id,
            ASPECT_HISTORY_COMMIT_CEILING,
        )?;
        Ok(super::aspect_history_projection::lineage_history(
            &envelopes,
            branch_id,
            lineage_id,
            &lineage_scope,
            &entity_scope,
            resolution,
            filter,
        ))
    }

    fn record_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        target: RecordRef,
        requested_target: HistoryAspectQueryTarget,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<AspectHistoryQueryResult, RelationalExecutionDenialCause> {
        let partition = match target {
            RecordRef::Entity(entity) => entity.partition_id,
            RecordRef::Relation(relation) => relation.partition_id,
        };
        self.demand_branch_history_regions(execution, branch_id, [partition])?;
        let envelopes = self.demand_branch_commit_envelopes(
            execution,
            branch_id,
            ASPECT_HISTORY_COMMIT_CEILING,
        )?;
        Ok(super::aspect_history_projection::record_history(
            &envelopes,
            branch_id,
            target,
            requested_target,
            filter,
        ))
    }

    fn lineage_scope(
        &self,
        start_lineage_id: &LineageId,
        traversed_event_ids: &[u64],
    ) -> BTreeSet<LineageId> {
        let mut scope = BTreeSet::from([*start_lineage_id]);
        for event in self
            .runtime
            .lineage
            .shared_events()
            .iter()
            .filter(|event| traversed_event_ids.contains(&event.event_id))
        {
            scope.extend(event.sources.iter().copied());
            scope.extend(event.targets.iter().copied());
        }
        scope
    }
}
