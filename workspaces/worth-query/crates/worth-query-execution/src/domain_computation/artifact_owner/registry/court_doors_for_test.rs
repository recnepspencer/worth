//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowArtifactRegistry {
    #[doc(hidden)]
    pub fn new_for_test(run_identity: String) -> Self {
        Self::new(run_identity)
    }
}

impl WorthQueryWorkflowArtifactRegistry {
    #[doc(hidden)]
    pub fn prepare_next_generation_for_test(
        self: &Arc<Self>,
    ) -> Result<WorthQueryArtifactProductionGenerationPending, WorthQueryArtifactDenial> {
        Self::prepare_next_generation(self)
    }
}
