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
        worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .admit_owned_bridge_async_request(declaration, selected)
    }

    pub fn retire_owned_bridge_async_request(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(), super::super::WorthQueryOwnedAsyncRuntimeDenial> {
        self.runtime.retire_owned_bridge_async_request(request)
    }

    pub fn advance_owned_bridge_async_request_to_timeout(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        coordinate: u64,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncTimeoutAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .advance_owned_bridge_async_request_to_timeout(request, coordinate)
    }

    pub fn schedule_owned_bridge_async_retry(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        timeout: &worth_runtime_bridge::facade::BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRetrySchedule,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .schedule_owned_bridge_async_retry(request, timeout)
    }

    pub fn advance_owned_bridge_async_retry(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        schedule: &worth_runtime_bridge::facade::BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRetryAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.runtime
            .advance_owned_bridge_async_retry(request, schedule, coordinate)
    }

    pub fn revalidate_owned_bridge_async_request(
        &self,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRevalidationAdmission,
        super::super::WorthQueryOwnedAsyncRuntimeDenial,
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
