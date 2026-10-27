use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub fn installed_owned_bridge_async_declaration(
        &self,
        identity: &crate::application::WorthQueryAsyncResourceRequestIdentity,
    ) -> Option<super::super::WorthQueryInstalledOwnedAsyncDeclaration> {
        self.runtime
            .installed_owned_bridge_async_declaration(identity)
    }

    pub fn admit_owned_bridge_async_request(
        &self,
        declaration: &super::super::WorthQueryInstalledOwnedAsyncDeclaration,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<
        super::super::WorthQueryOwnedAsyncRequestCustody,
        super::super::WorthQueryOwnedAsyncRuntimeStop,
    > {
        self.runtime
            .admit_owned_bridge_async_request(declaration, selected)
    }

    /// Selects the genuine observed head snapshot of a read-only source.
    pub fn admit_owned_bridge_async_request_at_branch_head(
        &self,
        declaration: &super::super::WorthQueryInstalledOwnedAsyncDeclaration,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<
        super::super::WorthQueryOwnedAsyncRequestCustody,
        super::super::WorthQueryOwnedAsyncRuntimeStop,
    > {
        self.runtime
            .admit_owned_bridge_async_request_at_branch_head(declaration, selected)
    }

    pub fn cleanup_owned_bridge_async_entry(
        &self,
        owner: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), super::super::WorthQueryOwnedAsyncRuntimeCause> {
        self.runtime.cleanup_owned_bridge_async_entry(owner)
    }

    pub fn cancel_custodied_owned_bridge_async_request(
        &self,
        owner: &super::super::WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), super::super::WorthQueryOwnedAsyncRuntimeCause> {
        self.runtime
            .cancel_custodied_owned_bridge_async_request(owner)
    }

    pub fn finish_owned_bridge_async_wake_cleanup(
        &self,
        owner: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), super::super::WorthQueryOwnedAsyncRuntimeCause> {
        self.runtime.finish_owned_bridge_async_wake_cleanup(owner)
    }

    pub fn retire_owned_bridge_async_request(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(), super::super::WorthQueryOwnedAsyncRuntimeDenial> {
        self.runtime.retire_owned_bridge_async_request(request)
    }

    pub fn advance_owned_bridge_async_request_to_timeout(
        &self,
        request: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
        coordinate: u64,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncTimeoutAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeCause,
    > {
        self.runtime
            .advance_owned_bridge_async_request_to_timeout(request, coordinate)
    }

    pub fn schedule_owned_bridge_async_retry(
        &self,
        request: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
        timeout: &worth_runtime_bridge::facade::BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRetrySchedule,
        super::super::WorthQueryOwnedAsyncRuntimeCause,
    > {
        self.runtime
            .schedule_owned_bridge_async_retry(request, timeout)
    }

    pub fn advance_owned_bridge_async_retry(
        &self,
        request: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
        schedule: &worth_runtime_bridge::facade::BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeAsyncRetryLineage,
        super::super::WorthQueryOwnedAsyncRuntimeCause,
    > {
        self.runtime
            .advance_owned_bridge_async_retry(request, schedule, coordinate)
    }

    pub fn revalidate_owned_bridge_async_request(
        &self,
        request: &mut super::super::WorthQueryOwnedAsyncRequestCustody,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRevalidation,
        super::super::WorthQueryOwnedAsyncRuntimeCause,
    > {
        self.runtime
            .revalidate_owned_bridge_async_request(request, selected)
    }

    pub fn owned_bridge_async_active_request_count(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, super::super::WorthQueryOwnedAsyncRuntimeDenial> {
        self.runtime
            .owned_bridge_async_active_request_count(request)
    }

    pub fn admit_owned_bridge_async_completion(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .admit_owned_bridge_async_completion(request, raw)
    }

    pub fn admit_owned_bridge_async_effects_indeterminate(
        &self,
        observation: worth_runtime_bridge::facade::BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .admit_owned_bridge_async_effects_indeterminate(observation)
    }

    pub fn order_owned_bridge_async_completion(
        &self,
        completion: &worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeMixedCauseOrdering,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime.order_owned_bridge_async_completion(completion)
    }

    pub fn owned_async_runtime_topology(
        &self,
    ) -> Option<super::super::WorthQueryOwnedAsyncRuntimeTopology> {
        self.runtime.owned_async_runtime_topology()
    }

    pub fn supersede_owned_bridge_async_live_view<T>(
        &mut self,
        view: &super::super::WorthQueryLiveView<T>,
        prior: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        displacing: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        super::super::WorthQueryAsyncResultTransitionBatch,
        super::super::WorthQueryAsyncSourceBindingError,
    > {
        self.runtime
            .supersede_owned_bridge_async_live_view(view, prior, displacing)
    }

    pub fn deny_owned_bridge_async_live_view<T>(
        &mut self,
        view: &super::super::WorthQueryLiveView<T>,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        super::super::WorthQueryAsyncResultTransitionBatch,
        super::super::WorthQueryAsyncSourceBindingError,
    > {
        self.runtime
            .deny_owned_bridge_async_live_view(view, request)
    }

    pub fn cancel_owned_bridge_async_live_view<T>(
        &mut self,
        view: &super::super::WorthQueryLiveView<T>,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        super::super::WorthQueryAsyncResultTransitionBatch,
        super::super::WorthQueryAsyncSourceBindingError,
    > {
        self.runtime
            .cancel_owned_bridge_async_live_view(view, request)
    }
}
