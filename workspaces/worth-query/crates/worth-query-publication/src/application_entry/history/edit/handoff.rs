//! Arm the edit receiver after actual selection, before checked delivery.
pub(super) use super::super::owner_handoff::OwnerHandoff;
pub use super::super::owner_handoff::WorthQueryHistoryOwnerHandoffPauseForTest as WorthQueryHistoryEditHandoffPauseForTest;
use super::*;

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryHistoryEdit<'_, S, H, I, P, A, Payload, Outputs>
where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    #[doc(hidden)]
    pub fn pause_after_history_adoption_for_test(
        &mut self,
    ) -> WorthQueryHistoryEditHandoffPauseForTest {
        let (receiver, control) = OwnerHandoff::arm(self.handoff.as_ref());
        self.handoff = Some(receiver);
        control
    }
}
