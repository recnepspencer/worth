//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowArtifactRegistry {
    #[doc(hidden)]
    pub fn new_for_test(run_identity: String) -> Self {
        Self::new(run_identity)
    }
}
