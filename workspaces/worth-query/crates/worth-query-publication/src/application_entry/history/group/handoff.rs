//! Pauses only after the group has stored the actual reservation, candidate or selection.
use super::super::owner_handoff::OwnerHandoff;
pub use super::super::owner_handoff::WorthQueryHistoryOwnerHandoffPauseForTest as WorthQueryHistoryGroupHandoffPauseForTest;
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryGroupHandoffPointForTest {
    Reservation,
    Candidate,
    Adoption,
}

impl<S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>
    WorthQueryGroupedHistoryDocument<'_, S, H, I, P, A, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    /// Rendezvous at the original Native receiver after endpoint capture,
    /// before its guarded adoption. This changes no operation outcome.
    #[doc(hidden)]
    pub fn pause_after_history_capture_for_test(
        &mut self,
    ) -> worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryCapturePauseForTest{
        self.document.owner.pause_after_history_capture_for_test()
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
    > WorthQueryHistoryGroupExecution<'_, S, H, I, P, A>
where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    #[doc(hidden)]
    pub fn pause_on_group_handoff_for_test(
        &mut self,
        point: WorthQueryHistoryGroupHandoffPointForTest,
    ) -> WorthQueryHistoryGroupHandoffPauseForTest {
        let previous = self.handoff.as_ref().map(|(_, receiver)| receiver);
        let (receiver, control) = OwnerHandoff::arm(previous);
        self.handoff = Some((point, receiver));
        control
    }
    pub(super) fn pause_at(&self, point: WorthQueryHistoryGroupHandoffPointForTest) {
        if let Some((selected, receiver)) = &self.handoff {
            if *selected == point {
                receiver.pause();
            }
        }
    }
}
