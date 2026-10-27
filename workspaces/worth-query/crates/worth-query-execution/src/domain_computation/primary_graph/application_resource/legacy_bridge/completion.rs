use super::{WorthQueryOwnedAsyncRuntimeDenial as Denial, WorthQueryOwnedAsyncRuntimeServices};
use worth_runtime_bridge::facade::{
    BridgeAsyncEffectsIndeterminateObservation, BridgeMixedCauseOrdering,
    BridgeOwnedAsyncCompletionAdmission, BridgeOwnedAsyncRequestAdmission,
};

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    /// Explicit retirement acts on the genuine immutable occurrence witness.
    pub fn retire_witness(&self, request: &BridgeOwnedAsyncRequestAdmission) -> Result<(), Denial> {
        self.bridge
            .retire_owned_async_request(request)
            .map(|_| ())
            .map_err(Denial::Completion)
    }

    /// Admission only. The existing LiveView consumer owns its publication handoff.
    pub fn admit_completion(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, Denial> {
        let validated = self
            .bridge
            .validate_owned_async_completion_envelope(request, raw)
            .map_err(Denial::Completion)?;
        self.bridge
            .admit_owned_async_completion(request, &validated)
            .map_err(Denial::Completion)
    }

    pub fn admit_effects_indeterminate(
        &self,
        observation: BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, Denial> {
        self.bridge
            .admit_owned_async_effects_indeterminate(observation)
            .map_err(Denial::Completion)
    }

    pub fn order_completion(
        &self,
        completion: &BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<BridgeMixedCauseOrdering, Denial> {
        self.bridge
            .order_owned_async_completion_report(completion)
            .map_err(Denial::Completion)
    }

    pub fn active_request_count(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, Denial> {
        self.bridge
            .owned_async_active_request_count(request)
            .map_err(Denial::Completion)
    }
}
