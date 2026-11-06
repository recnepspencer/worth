//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn retain_scope_selector_funded_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<WorthQueryObservedScopeSelector>, Stop> {
        Self::retain_scope_selector_funded(self, execution)
    }
}
