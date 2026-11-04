//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInstalledWorkflowArtifactContracts {
    #[doc(hidden)]
    pub fn empty_for_test() -> Self {
        Self::empty()
    }
}

impl WorthQueryInstalledWorkflowArtifactContracts {
    #[doc(hidden)]
    pub fn with_output_for_test(
        output: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    ) -> Self {
        Self::with_output(output)
    }
}
