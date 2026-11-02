//! Forward the real Native policy receiver; the SDK adds no independent rendezvous.
use super::*;
impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    #[doc(hidden)]
    pub fn pause_before_history_retention_change_for_test(&mut self) -> worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRetentionChangePauseForTest{
        self.owner.pause_before_history_retention_change_for_test()
    }
    #[doc(hidden)]
    pub fn pause_after_history_retention_change_for_test(&mut self) -> worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRetentionChangePauseForTest{
        self.owner.pause_after_history_retention_change_for_test()
    }
}
