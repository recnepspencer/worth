//! Forward the actual document page receiver's assembly rendezvous.
use super::*;
impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload>
{
    #[doc(hidden)]
    pub fn pause_after_history_page_assembly_for_test(&mut self) -> worth_query_execution::publication_boundary::document_history::WorthQueryHistoryPageAssemblyPauseForTest{
        self.owner.pause_after_history_page_assembly_for_test()
    }
}
