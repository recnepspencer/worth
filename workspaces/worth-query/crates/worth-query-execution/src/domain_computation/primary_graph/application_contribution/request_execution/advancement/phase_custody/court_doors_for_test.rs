//! Sealed test doors over this module's private members.
use super::*;

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    #[doc(hidden)]
    pub fn execution_for_for_test(
        &self, runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase> {
        Self::execution_for(self, runtime)
    }
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    #[doc(hidden)]
    pub fn test_host_request_for_test(&self) -> super::super::test_host_policy::TestHostRequest<'scope> {
        Self::test_host_request(self)
    }
}
