//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphApplicationAttemptStore {
    #[doc(hidden)]
    pub fn shared_observed_facts_for_test(
        &self, session: WorthQueryProviderSessionView<'_>) -> Option<(
        Arc<crate::domain_computation::primary_graph::ApplicationFactTable>,
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    )>{
        Self::shared_observed_facts(self, session)
    }

    pub fn reset_idle_replay_ordinals_for_test(&mut self) {
        assert!(
            self.attempts.is_empty(),
            "replay reset requires zero live application attempts"
        );
        self.next_registration = 0;
        self.next_overlay = 0;
    }
}
