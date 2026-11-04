//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderProgressionCompletion {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        lease: WorthQueryApplicationSnapshotLease,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        workflow_settlement_publication: Option<super::super::LocalWorkflowSettlementPublication>,
        request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryProgressedApplicationCommit {
        Self::finish(
            self,
            lease,
            running,
            workflow_settlement_publication,
            request,
        )
    }
}
