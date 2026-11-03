//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProgramActivationLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorthQueryProgramActivationLayout {
    #[doc(hidden)]
    pub fn program_revision_locator_for_test(&self) -> &AspectFieldLocator {
        &self.program_revision_locator
    }
}
