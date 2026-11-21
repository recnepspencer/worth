//! Compiled control doors arm their own provider and return registration custody.
use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::publication_fault_control::{
    WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Probe the actual request's maximum memory reservation at the next graph
    /// support retention. No refusal is fabricated; unaccounted requests continue.
    pub fn probe_next_discovery_support_retention_memory_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.runtime
            .primary_provider
            .publication_fault_control
            .probe_query_support_memory()
    }
    pub fn reject_next_commit_before_transaction_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.runtime
            .primary_provider
            .publication_fault_control
            .reject_pretransaction()
    }
    pub fn hold_post_commit_snapshot_failure_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.runtime
            .primary_provider
            .publication_fault_control
            .hold_snapshot_failure()
    }
}
