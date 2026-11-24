//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn graph_for_test(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.graph
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn receipt_basis_retention_for_test(
        &self,
    ) -> &Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>> {
        &self.receipt_basis_retention
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn attempts_for_test(
        &self,
    ) -> &Arc<Mutex<application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore>> {
        &self.attempts
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn application_attempt_operation_control_for_test(
        &self,
    ) -> &super::super::application_runtime::WorthQueryApplicationAttemptOperationControl {
        &self.application_attempt_operation_control
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn live_delivery_for_test(
        &self,
    ) -> &super::super::live_delivery::WorthQueryLiveDeliverySource {
        &self.live_delivery
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn application_attempt_resource_count_for_test(&self) -> usize {
        Self::application_attempt_resource_count(self)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn application_branch_commit_lane_for_test(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        Self::application_branch_commit_lane(self, observation)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn teardown_sentinel_for_test(
        &self,
    ) -> &Mutex<Option<Box<dyn std::any::Any + Send + Sync>>> {
        &self.teardown_sentinel
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn resolve_idempotency_binding_at_product_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        super::WorthQueryProviderIdempotencyResolution,
        super::WorthQueryProviderIdempotencyResolutionDenial,
    > {
        self.resolve_idempotency_binding_at_product(execution, binding, product)
    }

    #[doc(hidden)]
    pub fn committed_dispatch_outbox_count_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<usize, committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial> {
        use committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
        use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
        self.graph.with_runtime(execution, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .map_err(|_| Denial::ExactCommitUnavailable)?;
            let view = runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|_| Denial::ExactCommitUnavailable)?;
            let layout = self.graph.layout_ref().provider_dispatch_outbox();
            let outboxes = view
                .bounded_entities_of_kind(execution, layout.entity_kind, usize::MAX)
                .map_err(|_| Denial::Malformed)?
                .map_err(|_| Denial::Malformed)?;
            Ok(outboxes.records().len())
        })
    }
}
