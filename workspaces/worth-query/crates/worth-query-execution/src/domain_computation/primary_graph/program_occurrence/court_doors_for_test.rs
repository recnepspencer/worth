//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn complete_native_installation_for_test(
        &mut self, programs: Box<[Arc<super::super::application_installation::WorthQuerySupportedProgramRecord>]>) {
        Self::complete_native_installation(self, programs)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn installed_for_test( roster: Arc<WorthQueryProgramSupportRoster<Schema>>, activation: WorthQueryProgramActivationCell) -> Self {
        Self::installed(roster, activation)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn present_for_test(
        &self, revision: &ApplicationProgramRevision) -> Option<WorthQueryPresentedProgram<'_>> {
        Self::present(self, revision)
    }
}
