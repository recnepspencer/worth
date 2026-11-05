//! Sealed test doors over this module's private members.
use super::*;

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    #[doc(hidden)]
    pub fn execution_for_for_test(
        &self,
        runtime: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase> {
        Self::execution_for(self, runtime)
    }
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    #[doc(hidden)]
    pub fn test_host_request_for_test(
        &self,
    ) -> super::super::test_host_policy::TestHostRequest<'scope> {
        Self::test_host_request(self)
    }
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    #[doc(hidden)]
    pub fn execution_request_for_for_test(
        &self,
        runtime: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        Self::execution_request_for(self, runtime)
    }
}
