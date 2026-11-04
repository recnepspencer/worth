//! Sealed test doors over this module's private members.
use super::*;

impl CompiledWorkflowDefinition {
    #[doc(hidden)]
    pub fn resumed_at_for_test(&self, path: &str) -> Option<Self> {
        Self::resumed_at(self, path)
    }
}
