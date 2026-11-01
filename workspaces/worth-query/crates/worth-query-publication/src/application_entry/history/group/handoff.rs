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

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > WorthQueryHistoryGroupExecution<'_, S, H, I, P, A>
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
