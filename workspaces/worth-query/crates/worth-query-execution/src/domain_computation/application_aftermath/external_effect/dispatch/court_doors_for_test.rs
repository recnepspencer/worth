//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExternalEffectDispatch {
    #[doc(hidden)]
    pub fn matches_committed_owner_for_test(
        &self,
        runtime: crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity,
        committed: &crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
    ) -> bool {
        Self::matches_committed_owner(self, runtime, committed)
    }
}
