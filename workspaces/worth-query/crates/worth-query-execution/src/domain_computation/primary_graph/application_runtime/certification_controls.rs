use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Schedules one failure at the generic Query index-publication boundary.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn fail_next_index_publication_for_test(&self) {
        self.primary_provider.fail_next_index_publication_for_test();
    }

    /// Delays one output-readiness delivery before its unique World change is consumed.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        self.primary_provider
            .delay_next_output_readiness_delivery_for_test();
    }

    /// Observes the currently bound primary Bridge truth snapshot.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn primary_truth_snapshot_for_test(
        &self,
    ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.primary_provider
            .graph
            .current_truth_snapshot(&super::super::primary_truth_branch_identity())
    }

    /// Inspects the real Relational snapshot count and current branch basis.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn relational_snapshot_state_for_test(
        &self,
    ) -> (
        usize,
        worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    ) {
        self.primary_provider.graph.with_runtime_mut(|runtime| {
            let active = runtime.retention().inspect_plan().active_snapshot_count;
            let identity = runtime
                .branch_identity(self.relational_branch_identity.branch_id())
                .expect("the application branch remains owner registered");
            let (_, basis) = runtime
                .observe_branch(&identity)
                .expect("the application branch remains owner observable");
            (active, basis.descriptor().clone())
        })
    }

    #[cfg(test)]
    pub(crate) fn fail_next_durable_append_for_test(&self) {
        self.primary_provider
            .graph
            .with_runtime_mut(|runtime| runtime.fail_next_durable_append_for_test());
    }

    #[cfg(test)]
    pub(crate) fn provider_session_resource_count(&self) -> usize {
        self.primary_provider.application_attempt_resource_count()
    }

    #[cfg(test)]
    pub(crate) fn application_attempt_work(
        &self,
    ) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot
    {
        self.primary_provider.application_attempt_work()
    }
}
