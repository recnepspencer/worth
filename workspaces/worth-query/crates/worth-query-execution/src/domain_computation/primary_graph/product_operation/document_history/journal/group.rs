//! A group reserves one edge while allowing navigation to compete for selection.
use super::*;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn reserve_group(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: WorthQueryApplicationIdempotencyBinding,
        label: &str,
    ) -> Result<ReservedHistoryRecord, Denial> {
        let record = self.reserve(execution, meaning, label)?;
        self.pending = Some(PendingRecord::Group(record.node));
        Ok(record)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn owns_group(
        &self,
        record: &ReservedHistoryRecord,
    ) -> bool {
        matches!(self.pending,Some(PendingRecord::Group(node)) if node==record.node)
            && matches!(self.attempts.get(&record.key),
                Some(HistoryAttempt { binding: AttemptBinding::Reserved(node), .. }) if *node == record.node)
    }
}
