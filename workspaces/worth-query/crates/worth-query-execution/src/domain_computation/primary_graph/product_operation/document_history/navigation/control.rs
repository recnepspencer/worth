//! Rendezvous attached to the attempt's actual selection and delivery receivers.
use super::super::receiver_pause::ReceiverPauseGuard;
use super::{NavigationPhase, WorthQueryDocumentHistoryNavigation};
use std::time::Duration;

#[doc(hidden)]
pub struct WorthQueryHistoryNavigationPauseForTest {
    guard: ReceiverPauseGuard,
}

impl<S: worth_query_installation::facade::ApplicationSchema>
    WorthQueryDocumentHistoryNavigation<'_, S>
{
    #[doc(hidden)]
    pub fn candidate_branch_for_test(&self) -> Option<crate::basis::WorthQueryProductBranch> {
        let progress = self.attempt.progress.try_lock().ok()?;
        match &progress.phase {
            NavigationPhase::Candidate(candidate)
            | NavigationPhase::Publication { candidate, .. }
            | NavigationPhase::Unpublished { candidate, .. }
            | NavigationPhase::Selecting { candidate, .. } => Some(candidate.branch()),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn pause_before_history_navigation_selection_for_test(
        &mut self,
    ) -> WorthQueryHistoryNavigationPauseForTest {
        let mut progress = self
            .attempt
            .progress
            .try_lock()
            .expect("arm an idle attempt");
        WorthQueryHistoryNavigationPauseForTest {
            guard: ReceiverPauseGuard::arm(&mut progress.selection_pause),
        }
    }

    #[doc(hidden)]
    pub fn pause_after_history_navigation_adoption_for_test(
        &mut self,
    ) -> WorthQueryHistoryNavigationPauseForTest {
        let mut progress = self
            .attempt
            .progress
            .try_lock()
            .expect("arm an idle attempt");
        WorthQueryHistoryNavigationPauseForTest {
            guard: ReceiverPauseGuard::arm(&mut progress.delivery_pause),
        }
    }
}

impl WorthQueryHistoryNavigationPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.guard.wait_until_reached(timeout)
    }
    pub fn release(&self) {
        self.guard.release();
    }
}
