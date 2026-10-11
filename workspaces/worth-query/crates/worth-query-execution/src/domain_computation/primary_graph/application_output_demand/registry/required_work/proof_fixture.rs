//! Real membership observation for the acknowledgement writer's basis witness.
use super::*;
impl SelectedRequiredWork {
    /// Queue-state fixture only: this does not prove a graph row Current.
    pub(in crate::domain_computation::primary_graph) fn acknowledge_fixture(
        self,
    ) -> Option<ReplacedRequiredWorkHint> {
        self.consume_selected_cause()
    }
}
impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) fn fixture_selected_work(
        &self,
    ) -> SelectedRequiredWork {
        let queue = self.queue.upgrade().expect("the registry owns its queue");
        let RequiredWorkPop::Claimed(claim) = queue.pop() else {
            panic!("the marked row is queued")
        };
        let work = claim
            .select_without_advance()
            .expect("the actual claim selects a live cause");
        assert_eq!(
            work.key(),
            self.key.as_ref(),
            "the test observes its own row"
        );
        work
    }
    pub(in crate::domain_computation::primary_graph) fn fixture_is_unresolved(&self) -> bool {
        self.state.lock().unwrap().unresolved_initial
    }
}
