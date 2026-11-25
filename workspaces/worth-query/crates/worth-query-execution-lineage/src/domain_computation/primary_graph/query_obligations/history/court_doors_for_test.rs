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

    /// Passively inspects the already-acknowledged folded ledger for `identity` without
    /// calling `finish_pending` or mutating any staged publication.
    #[doc(hidden)]
    pub fn acknowledged_original_operation_for_test(
        &self,
        identity: worth_foundational::facade::ProgramSupportObligationIdentity,
    ) -> Option<journal::QueryOriginalOperationObligation> {
        let medium = self.medium.get()?;
        if identity.home() != medium.home() {
            return None;
        }
        self.state()
            .journal
            .as_ref()
            .and_then(|journal| {
                worth_query_execution_obligations::facade::ledger_inspect::original_operations(
                    &journal.ledger,
                )
                .get(&identity)
            })
            .cloned()
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
