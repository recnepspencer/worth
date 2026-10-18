//! Compiled control doors arm their own provider and return registration custody.
use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::publication_fault_control::{
    WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn reject_next_commit_before_transaction_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.primary_provider
            .publication_fault_control
            .reject_pretransaction()
    }
    pub fn hold_post_commit_snapshot_failure_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.primary_provider
            .publication_fault_control
            .hold_snapshot_failure()
    }
}
