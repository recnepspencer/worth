//! Forward rendezvous to the original navigation attempt receiver.
use super::*;
pub use worth_query_execution::publication_boundary::document_history::WorthQueryHistoryNavigationPauseForTest;

impl<S, H, N, P> WorthQueryHistoryNavigation<'_, '_, S, H, N, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
{
    #[doc(hidden)]
    pub fn candidate_branch_for_test(
        &self,
    ) -> Option<worth_query_execution::facade::product::WorthQueryProductBranch> {
        self.navigation.candidate_branch_for_test()
    }

    #[doc(hidden)]
    pub fn pause_before_history_navigation_selection_for_test(
        &mut self,
    ) -> WorthQueryHistoryNavigationPauseForTest {
        self.navigation
            .pause_before_history_navigation_selection_for_test()
    }

    #[doc(hidden)]
    pub fn pause_after_history_navigation_adoption_for_test(
        &mut self,
    ) -> WorthQueryHistoryNavigationPauseForTest {
        self.navigation
            .pause_after_history_navigation_adoption_for_test()
    }
}
