//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProgramSupportLifecycle {
    #[doc(hidden)]
    pub fn begin_retirement_for_test(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramSupportRetirementAttempt, WorthQueryProgramSupportRetirementDenial>
    {
        Self::begin_retirement(self, revision)
    }
}
