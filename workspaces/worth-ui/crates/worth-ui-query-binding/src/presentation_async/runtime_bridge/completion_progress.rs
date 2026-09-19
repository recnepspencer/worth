use worth_query::facade::runtime;

use super::{
    WorthUiPresentationAsyncObservation, WorthUiPresentationCompletionAdvance,
    WorthUiPresentationCompletionDenial, WorthUiPresentationRuntimeAdmission,
};

#[derive(Debug)]
pub(crate) struct WorthUiPresentationCompletionProgress {
    report: worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission,
    ordering: Option<worth_runtime_bridge::facade::BridgeMixedCauseOrdering>,
    batch: Option<runtime::WorthQueryAsyncResultTransitionBatch>,
    observation: Option<WorthUiPresentationAsyncObservation>,
}

impl WorthUiPresentationRuntimeAdmission {
    pub(crate) fn begin_owner_validated_completion(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
        payload_byte_len: u64,
    ) -> Result<WorthUiPresentationCompletionProgress, WorthUiPresentationCompletionDenial> {
        let raw = self.owner_completion_envelope(payload_byte_len);
        let report = workspace
            .admit_owned_bridge_async_completion(&self.request, raw)
            .map_err(|error| WorthUiPresentationCompletionDenial::QueryOwned(Box::new(error)))?;
        Ok(WorthUiPresentationCompletionProgress::new(report))
    }

    pub(crate) fn begin_owner_effects_indeterminate(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
        payload_byte_len: u64,
    ) -> Result<WorthUiPresentationCompletionProgress, WorthUiPresentationCompletionDenial> {
        let observation = self.effects_indeterminate_issuer.certify(payload_byte_len);
        let report = workspace
            .admit_owned_bridge_async_effects_indeterminate(observation)
            .map_err(|error| WorthUiPresentationCompletionDenial::QueryOwned(Box::new(error)))?;
        Ok(WorthUiPresentationCompletionProgress::new(report))
    }

    pub(crate) fn resume_completion(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
        progress: &mut WorthUiPresentationCompletionProgress,
    ) -> Result<WorthUiPresentationCompletionAdvance, WorthUiPresentationCompletionDenial> {
        if progress.ordering.is_none() {
            progress.ordering = Some(
                workspace
                    .order_owned_bridge_async_completion(&progress.report)
                    .map_err(|error| {
                        WorthUiPresentationCompletionDenial::QueryOwned(Box::new(error))
                    })?,
            );
        }
        if progress.batch.is_none() {
            progress.batch = Some(
                self.admit_transitions(
                    workspace,
                    progress
                        .ordering
                        .as_ref()
                        .expect("completion ordering is retained before transition admission"),
                )
                .map_err(|error| {
                    WorthUiPresentationCompletionDenial::QueryTransition(Box::new(error))
                })?,
            );
        }
        if progress.observation.is_none() {
            progress.observation = Some(self.observation(workspace).map_err(|error| {
                WorthUiPresentationCompletionDenial::Observation(Box::new(error))
            })?);
        }
        Ok(WorthUiPresentationCompletionAdvance {
            observation: progress
                .observation
                .expect("completion observation is retained before completion"),
        })
    }

    fn owner_completion_envelope(
        &self,
        payload_byte_len: u64,
    ) -> worth_signal::facade::RawCompletionEnvelope {
        let descriptor = self
            .request
            .request()
            .lowered()
            .resource_descriptor()
            .expect("presentation request retains its resource descriptor");
        worth_signal::facade::RawCompletionEnvelope::new(
            self.request.request().request_handle().request_id(),
            self.request.request().request_handle().generation(),
            self.request.request().request_handle().branch_epoch(),
            self.request.request().attempt(),
            descriptor.payload_contract_digest().clone(),
            payload_byte_len,
        )
    }
}

impl WorthUiPresentationCompletionAdvance {
    pub const fn observation(&self) -> WorthUiPresentationAsyncObservation {
        self.observation
    }
}

impl WorthUiPresentationCompletionProgress {
    fn new(report: worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission) -> Self {
        Self {
            report,
            ordering: None,
            batch: None,
            observation: None,
        }
    }
}
