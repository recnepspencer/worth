//! Forward only the document receiver's actual capture/adoption rendezvous.
use super::*;
pub use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryCapturePauseForTest;

impl<S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>>
    WorthQueryHistoryEdit<'_, S, H, I, P, A, Payload>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    #[doc(hidden)]
    pub fn pause_after_history_capture_for_test(
        &mut self,
    ) -> WorthQueryDocumentHistoryCapturePauseForTest {
        self.document.owner.pause_after_history_capture_for_test()
    }
}
