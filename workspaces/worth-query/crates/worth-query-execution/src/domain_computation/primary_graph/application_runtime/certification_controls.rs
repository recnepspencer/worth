use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryPrimaryGraphApplicationRuntime;
#[cfg(feature = "test-primary-graph-faults")]
mod native_field_write;
#[cfg(feature = "test-primary-graph-faults")]
mod workflow_approval;
#[cfg(feature = "test-primary-graph-faults")]
mod workflow_definition;
#[cfg(feature = "test-primary-graph-faults")]
mod workflow_nodes;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Schedules failure at the real post-commit snapshot admission boundary.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn fail_next_post_commit_snapshot_for_test(&self) {
        self.primary_provider
            .fail_next_post_commit_snapshot_for_test();
    }

    /// Holds the next producer commit's rebased facts under a requirement
    /// to verify them in full, where the commit would seal them as exact.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn leave_next_producer_settlement_unsealed_for_test(&self) {
        self.primary_provider
            .leave_next_producer_settlement_unsealed_for_test();
    }

    /// Drops only rebuildable workflow-instance progress projections.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn release_workflow_instance_progress_for_test(&self) {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .release_workflow_instance_progress();
    }

    /// Evicts every compiled workflow definition, leaving revision pins intact.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn release_workflow_compilation_for_test(&self) {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .with_workflow_compilation_reuse_mut(|reuse| reuse.release_all());
    }

    /// Delays one output-readiness delivery before its unique World change is consumed.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        self.primary_provider
            .delay_next_output_readiness_delivery_for_test();
    }

    /// Fails one readiness evaluation after performed-change delivery.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn fail_next_output_readiness_evaluation_for_test(&self) {
        self.primary_provider
            .fail_next_output_readiness_evaluation_for_test();
    }

    /// Holds real World observations while the next ready output opens a read.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn press_next_ready_read_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_ready_read_with_world_snapshots_for_test();
    }

    /// Holds real World observations while the next readiness evaluation selects its basis.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_readiness_with_world_snapshots_for_test();
    }

    /// Counts readiness decisions actually attempted by this application runtime.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn output_readiness_attempt_count_for_test(&self) -> u64 {
        self.next_output_producer_attempt
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Counts post-publication checkpoints and any World snapshots retained inside their receipts.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn output_checkpoint_snapshot_state_for_test(&self) -> (usize, usize) {
        self.output_demands.output_checkpoint_snapshot_state()
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn hold_world_snapshot_pressure_for_test(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Vec<crate::basis::WorthQueryProductBranchLease> {
        let mut held = Vec::new();
        let mut exhausted = false;
        for _ in 0..1_024 {
            match self.product_runtime.integration_admit_product_branch(branch) {
                Ok(observation) => held.push(observation),
                Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted) => {
                    exhausted = true;
                    break;
                }
                Err(error) => panic!("World snapshot pressure failed for another reason: {error:?}"),
            }
        }
        assert!(
            exhausted,
            "snapshot pressure must reach actual World capacity"
        );
        held
    }

    /// Counts performed sources awaiting their required-output admission owner.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn prepared_required_output_source_count_for_test(&self) -> usize {
        self.output_demands.prepared_source_count()
    }

    /// Counts retained source custody, including consumed roots awaiting program completion.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn retained_source_custody_count_for_test(&self) -> usize {
        self.output_demands.retained_source_custody_count()
    }

    /// Observes the currently bound primary Bridge truth snapshot.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn primary_truth_snapshot_for_test(
        &self,
    ) -> Result<
        Option<worth_runtime_bridge::facade::TruthSnapshotIdentity>,
        crate::facade::primary_graph::WorthQueryHandleDenial,
    > {
        self.primary_provider
            .graph
            .current_truth_snapshot(&super::super::primary_truth_branch_identity())
    }

    /// Inspects the real Relational snapshot count and current branch basis.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn relational_snapshot_state_for_test(
        &self,
    ) -> Result<
        (
            usize,
            worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        ),
        crate::facade::primary_graph::WorthQueryHandleDenial,
    > {
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

    #[doc(hidden)]
    #[cfg(any(test, feature = "test-durability-faults"))]
    pub fn fail_next_durable_append_for_test(
        &self,
    ) -> Result<(), crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.primary_provider
            .graph
            .with_runtime_mut(|runtime| runtime.fail_next_durable_append_for_test())
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
