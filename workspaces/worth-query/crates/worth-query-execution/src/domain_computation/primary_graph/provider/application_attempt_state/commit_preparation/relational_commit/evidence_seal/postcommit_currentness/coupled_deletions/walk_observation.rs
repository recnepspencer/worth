//! Test observer method on `WorthQueryPrimaryGraphApplicationRuntime`.
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn coupled_commit_patches_walked_on_this_thread_for_test(&self) -> u64 {
        worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::coupled_deletions::walk_observation::walked()
    }
}
