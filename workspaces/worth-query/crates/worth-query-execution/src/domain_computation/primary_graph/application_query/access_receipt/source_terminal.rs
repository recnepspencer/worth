//! Only a genuine completed receipt can surrender its Query terminal.
use super::WorthQueryApplicationQueryAccessReceipt;
use crate::domain_computation::{
    primary_graph::source_facts::SourceFactsAllocation,
    provider_session::WorthQueryCompletedSourceRead,
};
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl WorthQueryApplicationQueryAccessReceipt {
    pub(in crate::domain_computation::primary_graph::application_query) fn into_managed_source_terminal(
        self,
        context: crate::domain_computation::provider_session::WorthQueryQuerySourceContext,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
        work: impl FnOnce(u64) -> Result<(), Cause>,
    ) -> Result<Option<WorthQueryCompletedSourceRead>, Cause> {
        self.read_completion
            .into_managed_source(context, execution, allocation, work)
    }
}
