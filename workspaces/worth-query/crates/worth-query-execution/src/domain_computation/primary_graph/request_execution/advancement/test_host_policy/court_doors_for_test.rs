//! Sealed test doors over this module's private members.
use super::*;

impl<'scope> TestHostRequest<'scope> {
    #[doc(hidden)]
    pub fn execution_request_for_for_test(
        &self,
        runtime: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        Self::execution_request_for(self, runtime)
    }
}

impl<'scope> TestHostRequest<'scope> {
    #[doc(hidden)]
    pub fn phase_for_application_for_test(
        &self,
        runtime: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::phase_for_application(self, runtime)
    }
}

impl<'scope> TestHostRequest<'scope> {
    #[doc(hidden)]
    pub fn test_host_request_for_test(&self) -> Self {
        Self::test_host_request(self)
    }
}

impl<'scope> TestHostRequest<'scope> {
    #[doc(hidden)]
    pub fn phase_for_direct_for_test(
        &self,
        run: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::phase_for_direct(self, run)
    }
}

impl<'scope> TestHostRequest<'scope> {
    #[doc(hidden)]
    pub fn phase_for_workflow_for_test(
        &self,
        run: &(impl super::super::source::WorthQueryAdvancementSource + ?Sized),
    ) -> WorthQueryAdvancementPhase<'scope> {
        Self::phase_for_workflow(self, run)
    }
}
