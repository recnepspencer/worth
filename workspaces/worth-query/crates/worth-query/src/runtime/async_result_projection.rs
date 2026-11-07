use std::collections::BTreeMap;

use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::async_result_state::{
    WorthQueryRuntimeAsyncResultProjection, WorthQueryRuntimeAsyncResultState,
};
use super::{
    WorthQueryLiveArtifactTarget, WorthQueryRuntimeError, WorthQueryRuntimeLiveSubscriptionState,
};

fn project_live_async_result_state(
    live_subscriptions: &mut BTreeMap<
        WorthQueryLiveArtifactTarget,
        WorthQueryRuntimeLiveSubscriptionState,
    >,
    view_name: &str,
    projection: &WorthQueryRuntimeAsyncResultProjection,
    basis_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryRuntimeAsyncResultState, WorthQueryRuntimeError> {
    let target = WorthQueryLiveArtifactTarget::from_view_name(view_name);
    let state = live_subscriptions
        .get_mut(&target)
        .ok_or_else(|| WorthQueryRuntimeError::MissingLiveSubscription(view_name.to_string()))?;
    let expected_basis = state.async_source_binding.as_ref().map(
        super::async_source_binding::WorthQueryRuntimeAsyncSourceBinding::current_basis_identity,
    );
    let expected_checkpoint = state.async_source_binding.as_ref().map(
        super::async_source_binding::WorthQueryRuntimeAsyncSourceBinding::current_generation_identity,
    );
    let expected_basis = expected_basis
        .as_ref()
        .unwrap_or_else(|| state.installation.basis_binding_identity());
    let expected_checkpoint = expected_checkpoint
        .as_ref()
        .unwrap_or_else(|| state.active_lane_handle.checkpoint_identity());
    let kind = projection.kind();
    if basis_identity != expected_basis && !kind.permits_basis_or_generation_drift() {
        return Err(async_result_state_error(
            view_name,
            "PreviewBasisMismatchRequiresTypedState",
        ));
    }
    if checkpoint_identity != expected_checkpoint && !kind.permits_basis_or_generation_drift() {
        return Err(async_result_state_error(
            view_name,
            "GenerationDriftRequiresTypedState",
        ));
    }

    let async_result_state = WorthQueryRuntimeAsyncResultState::new(
        kind,
        projection.causality_identity(),
        basis_identity,
        checkpoint_identity,
    );
    state.async_result_state = Some(async_result_state.clone());
    Ok(async_result_state)
}

fn async_result_state_error(view_name: &str, message: &str) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage: "async-result-state",
        message: message.to_string(),
    }
}

impl super::WorthQueryRuntime {
    pub(crate) fn project_async_result_state(
        &mut self,
        view_name: &str,
        projection: &WorthQueryRuntimeAsyncResultProjection,
        basis_identity: &WorthQueryEvidenceIdentity,
        checkpoint_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<WorthQueryRuntimeAsyncResultState, WorthQueryRuntimeError> {
        project_live_async_result_state(
            &mut self.live_subscriptions,
            view_name,
            projection,
            basis_identity,
            checkpoint_identity,
        )
    }
}
