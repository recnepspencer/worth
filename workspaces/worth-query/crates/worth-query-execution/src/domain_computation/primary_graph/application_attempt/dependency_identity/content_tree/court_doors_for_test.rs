//! Sealed test doors over this module's private members.
use super::*;

impl CompletedFactRoutes {
    #[doc(hidden)]
    pub fn from_facts_for_test(
        body: FactPayload,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_facts(body, control)
    }
}

impl CompletedFactRoutes {
    #[doc(hidden)]
    pub fn matches_body_for_test(&self, body: &FactPayload) -> bool {
        Self::matches_body(self, body)
    }
}
