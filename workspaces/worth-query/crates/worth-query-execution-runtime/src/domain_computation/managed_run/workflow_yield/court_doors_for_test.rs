//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryYieldedWorkflowRun {
    #[doc(hidden)]
    pub fn artifacts_for_test(&self) -> &WorthQueryFrozenWorkflowArtifactAuthority {
        &self.artifacts
    }
}
