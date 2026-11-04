//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn capture_application_read_set_for_test(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        Self::capture_application_read_set(self, _execution, authority, control)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn compare_application_read_set_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        Self::compare_application_read_set(self, execution, authority, receipt, control)
    }
}

impl<'provider> PrimaryDecisionReadSetComparison<'provider> {
    #[doc(hidden)]
    pub fn acquire_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &'provider WorthQueryPrimaryGraphProvider,
        session: WorthQueryProviderSessionView<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        Self::acquire(execution, provider, session, control)
    }
}

impl<'provider> PrimaryDecisionReadSetComparison<'provider> {
    #[doc(hidden)]
    pub fn compare_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        Self::compare(self, execution, session, evidence, admission, control)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn recompare_application_read_set_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        fresh: WorthQueryFreshDecisionReadSet,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        Self::recompare_application_read_set(self, execution, authority, fresh, control)
    }
}
