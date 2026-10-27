use crate::history::data::{LineageAspectHistory, LineageAspectHistoryQueryResult};
use crate::lineage::access::LineageAccess;
use crate::lineage::data::HistoricalResolutionRequest;
use crate::visibility::materialization::read_records::ProjectionAspectFilter;

impl<'runtime> LineageAccess<'runtime> {
    pub fn entity_aspect_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: HistoricalResolutionRequest,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<Option<LineageAspectHistory>, crate::execution::RelationalExecutionDenialCause>
    {
        Ok(self
            .entity_aspect_history_with_trace(execution, request, filter)?
            .history)
    }

    pub fn entity_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: HistoricalResolutionRequest,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<LineageAspectHistoryQueryResult, crate::execution::RelationalExecutionDenialCause>
    {
        self.runtime
            .history()
            .lineage_entity_aspect_history_with_trace(
                execution,
                &request.branch_id,
                request.lineage_id,
                filter,
            )
    }
}
