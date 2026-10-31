//! Forward the actual physical-context receiver's final-guard rendezvous.
use super::super::*;
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryDocument<'_, S, H>
{
    #[doc(hidden)]
    pub fn pause_before_history_context_refresh_for_test(&mut self) -> worth_query_execution::publication_boundary::document_history::WorthQueryHistoryContextRefreshPauseForTest{
        self.owner.pause_before_history_context_refresh_for_test()
    }
}
