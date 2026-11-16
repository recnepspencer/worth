//! Read-only test access to the actual installed Directory capability guard.
use super::*;
impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn admit_directory_capability_for_test(
        &self,
        capability: crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        self.admit_directory_capability(capability)
    }
}
