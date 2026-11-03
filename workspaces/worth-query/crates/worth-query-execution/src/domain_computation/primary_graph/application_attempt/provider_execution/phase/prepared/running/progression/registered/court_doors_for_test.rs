//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
    ) {
        (self.staged, self.steps, self.dispatch_outbox)
    }
}
