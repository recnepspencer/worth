//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn complete_native_installation_for_test<
        P: crate::ports::SupportedProgramRecordPort + 'static,
    >(
        &mut self,
        programs: Box<[Arc<P>]>,
    ) {
        let erased: Vec<Arc<dyn crate::ports::SupportedProgramRecordPort>> = programs
            .into_vec()
            .into_iter()
            .map(|program| program as Arc<dyn crate::ports::SupportedProgramRecordPort>)
            .collect();
        Self::complete_native_installation(self, erased.into_boxed_slice())
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn installed_for_test(
        roster: Arc<WorthQueryProgramSupportRoster<Schema>>,
        activation: WorthQueryProgramActivationCell,
    ) -> Self {
        Self::installed(roster, activation)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn present_for_test(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryPresentedProgram<'_>> {
        Self::present(self, revision)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn present_for_commit_for_test(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryPresentedProgram<'_>, WorthQueryApplicationCommitDenial> {
        Self::present_for_commit(self, revision)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn retain_interpretation_for_test(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportInterpretation> {
        Self::retain_interpretation(self, revision)
    }
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    #[doc(hidden)]
    pub fn rostered_for_rendering_for_test(
        &self,
        rendering: &AspectValue,
    ) -> Option<&WorthQueryProgramSupportEntry> {
        Self::rostered_for_rendering(self, rendering)
    }
}
