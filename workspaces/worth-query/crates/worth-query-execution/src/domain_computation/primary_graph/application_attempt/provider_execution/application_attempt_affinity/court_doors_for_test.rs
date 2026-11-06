//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationAttemptBasis {
    #[doc(hidden)]
    pub fn affinity_mismatches_for_test(
        &self,
        provider_session: &WorthQueryProviderSessionTerminalBinding,
    ) -> std::collections::BTreeSet<WorthQueryApplicationAttemptAffinityMismatch> {
        Self::affinity_mismatches(self, provider_session)
    }
}

impl WorthQueryApplicationAttemptBasis {
    #[doc(hidden)]
    pub fn affinity_mismatches_view_for_test(
        &self,
        plan: &impl WorthQueryApplicationAttemptAffinityView,
    ) -> std::collections::BTreeSet<WorthQueryApplicationAttemptAffinityMismatch> {
        Self::affinity_mismatches_view(self, plan)
    }
}

impl WorthQueryApplicationAttemptBasis {
    #[doc(hidden)]
    pub fn take_original_completion_for_test(
        &mut self,
    ) -> crate::facade::primary_graph::OriginalCompletionReceiver {
        self.take_original_completion()
    }
}
