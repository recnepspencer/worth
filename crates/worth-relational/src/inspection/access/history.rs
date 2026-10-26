use crate::history::data::{AspectHistoryQueryResult, BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, RelationId};
use crate::lineage::data::HistoricalLineageResolution;
use crate::visibility::materialization::read_records::ProjectionAspectFilter;

use super::InspectionAccess;

impl<'runtime> InspectionAccess<'runtime> {
    pub(crate) fn recent_commit_ids(
        &self,
        branch_id: Option<&BranchId>,
        limit: usize,
    ) -> Vec<CommitId> {
        self.runtime.history().recent_commit_ids(branch_id, limit)
    }

    pub(crate) fn branch_head_ref(&self, branch_id: &BranchId) -> Option<RelationalCommitReceipt> {
        self.runtime.history().branch_head(branch_id)
    }

    pub(crate) fn resolve_lineage_record_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        entity_id: EntityId,
    ) -> Result<Option<HistoricalLineageResolution>, crate::execution::RelationalExecutionDenialCause>
    {
        self.runtime.lineage_access().resolve_record_history(execution,
            crate::facade::lineage::RecordHistoryRequest {
                branch_id: branch_id.clone(),
                entity_id,
                boundedness_basis:
                    crate::facade::lineage::HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
        )
    }

    pub(crate) fn entity_aspect_history_with_trace(
        &self,
        branch_id: &BranchId,
        entity_id: EntityId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> AspectHistoryQueryResult {
        self.runtime
            .history()
            .entity_aspect_history_with_trace(branch_id, entity_id, filter)
    }

    pub(crate) fn relation_aspect_history_with_trace(
        &self,
        branch_id: &BranchId,
        relation_id: RelationId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> AspectHistoryQueryResult {
        self.runtime
            .history()
            .relation_aspect_history_with_trace(branch_id, relation_id, filter)
    }
}
