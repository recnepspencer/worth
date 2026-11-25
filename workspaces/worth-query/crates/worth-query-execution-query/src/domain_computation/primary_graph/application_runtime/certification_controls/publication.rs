//! Compiled control doors arm their own provider and return registration custody.
use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::publication_fault_control::{
    WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
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
            .publication_fault_control_ref()
            .probe_query_support_memory()
    }
    /// Loses the next application commit's answer after the real commit
    /// resolves, whatever World did, so its decision is indeterminate (R9).
    /// It cannot commit, publish, release or classify anything.
    pub fn lose_next_commit_answer_for_test(&self) {
        self.runtime
            .primary_provider
            .lose_next_commit_answer_for_test();
    }
    /// Parks the next application commit after World resolved it and before
    /// its answer is delivered to the caller.
    pub fn pause_after_commit_before_answer_for_test(
        &self,
    ) -> crate::domain_computation::primary_graph::provider::WorthQueryCommitAnswerPauseForTest
    {
        self.runtime
            .primary_provider
            .pause_after_commit_before_answer_for_test()
    }
    pub fn reject_next_commit_before_transaction_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.runtime
            .primary_provider
            .publication_fault_control_ref()
            .reject_pretransaction()
    }
    pub fn hold_post_commit_snapshot_failure_for_test(
        &self,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.runtime
            .primary_provider
            .publication_fault_control_ref()
            .hold_snapshot_failure()
    }
}
