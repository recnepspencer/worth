//! Sealed test doors over this module's private members.
use super::*;

impl QueryObligationHistory {
    #[doc(hidden)]
    pub fn begin_retirement_for_test(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramSupportRetirementAttempt, WorthQueryProgramSupportRetirementDenial>
    {
        Self::begin_retirement(self, revision)
    }
}

impl WorthQueryProgramSupportRetirementAttempt {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        current_branches: usize,
        retained_program_bytes: usize,
    ) -> Result<WorthQueryProgramSupportRetirementReceipt, WorthQueryProgramSupportRetirementDenial>
    {
        Self::finish(self, current_branches, retained_program_bytes)
    }
}
