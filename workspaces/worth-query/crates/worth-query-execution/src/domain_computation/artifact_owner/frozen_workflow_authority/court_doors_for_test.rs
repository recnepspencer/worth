//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryFrozenWorkflowArtifactAuthority {
    #[doc(hidden)]
    pub fn registry_for_test(&self) -> Arc<WorthQueryWorkflowArtifactRegistry> {
        Self::registry(self)
    }
}
