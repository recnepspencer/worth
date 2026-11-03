//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphApplicationAttemptStore {
    #[doc(hidden)]
    pub fn shared_observed_facts_for_test(
        &self, session: WorthQueryProviderSessionView<'_>) -> Option<(
        Arc<crate::domain_computation::primary_graph::ApplicationFactTable>,
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    )> {
        Self::shared_observed_facts(self, session)
    }
}
