use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::{
    WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError,
    WorthQueryAsyncSourceBindingErrorKind, WorthQueryLiveArtifactTarget, WorthQueryLiveView,
    WorthQueryRuntime, WorthQueryRuntimeAsyncResultState, WorthQueryRuntimeAsyncResultStateKind,
};

impl WorthQueryRuntime {
    pub fn supersede_owned_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        prior: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        displacing: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        let product = self.installed_product.as_ref().ok_or_else(|| {
            foreign_request("owned async supersession requires an installed product runtime")
        })?;
        let supersession = product
            .conditional
            .admit_owned_async_supersession(prior, displacing)
            .map_err(|denial| foreign_request(denial.detail().to_owned()))?;
        let prior_request = supersession.prior();
        let displacing_request = supersession.displacing();
        let runtime_provenance = self.runtime_provenance();
        let target = WorthQueryLiveArtifactTarget::from_view_name(view.name());
        let state = self.live_subscriptions.get_mut(&target).ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingLiveSubscription,
                format!("live view `{}` has no active subscription", view.name()),
            )
        })?;
        let current = state
            .async_result_state
            .as_ref()
            .map(WorthQueryRuntimeAsyncResultState::kind);
        if !matches!(
            current,
            Some(
                WorthQueryRuntimeAsyncResultStateKind::Pending
                    | WorthQueryRuntimeAsyncResultStateKind::Current
                    | WorthQueryRuntimeAsyncResultStateKind::Unresolved
                    | WorthQueryRuntimeAsyncResultStateKind::Superseded
            )
        ) {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::IllegalResultTransition,
                format!("live view `{}` is no longer pending", view.name()),
            ));
        }
        let binding = state.async_source_binding.as_ref().ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
                format!("live view `{}` has no async binding", view.name()),
            )
        })?;
        if binding.declaration_identity_reference()
            != prior_request.lowered().declaration_identity()
            || binding.current_request_identity_reference() != prior_request.request_identity()
        {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::ForeignRequest,
                format!(
                    "live view `{}` is not bound to the declaration being superseded",
                    view.name()
                ),
            ));
        }
        if current == Some(WorthQueryRuntimeAsyncResultStateKind::Superseded) {
            return retained_terminal_batch(runtime_provenance, view, state);
        }
        let basis = binding.current_basis_identity();
        let checkpoint = binding.current_generation_identity();
        let binding_identity = binding.binding_identity().clone();
        let causality =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_owned_async_supersession_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("live_target"), view.name())
                .field_shape(
                    WorthQueryEvidenceTag::new("prior_request"),
                    prior_request.request_identity_for_reporting(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("displacing_request"),
                    displacing_request.request_identity_for_reporting(),
                )
                .seal();
        let superseded = WorthQueryRuntimeAsyncResultState::new(
            WorthQueryRuntimeAsyncResultStateKind::Superseded,
            &causality,
            &basis,
            &checkpoint,
        );
        state.async_result_state = Some(superseded.clone());
        let remask_posture = state.remask_posture.clone();
        Ok(WorthQueryAsyncResultTransitionBatch::admitted(
            runtime_provenance,
            view.name(),
            binding_identity,
            basis,
            checkpoint,
            remask_posture,
            vec![superseded],
            0,
        ))
    }

    pub fn deny_owned_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        self.transition_owned_bridge_async_live_view(
            view,
            request,
            WorthQueryRuntimeAsyncResultStateKind::Denied,
            "worth_query_owned_async_before_effects_denial_v1",
            "denied_request",
        )
    }

    pub fn cancel_owned_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        self.transition_owned_bridge_async_live_view(
            view,
            request,
            WorthQueryRuntimeAsyncResultStateKind::Cancelled,
            "worth_query_owned_async_before_effects_cancellation_v1",
            "cancelled_request",
        )
    }

    fn transition_owned_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        terminal_kind: WorthQueryRuntimeAsyncResultStateKind,
        evidence_family: &'static str,
        request_field: &'static str,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        let admitted = self
            .installed_product
            .as_ref()
            .ok_or_else(|| foreign_request("owned async transition requires an installed product"))?
            .conditional
            .validate_owned_async_request_occurrence(request)
            .map_err(|denial| foreign_request(denial.detail()))?;
        let runtime_provenance = self.runtime_provenance();
        let target = WorthQueryLiveArtifactTarget::from_view_name(view.name());
        let state = self.live_subscriptions.get_mut(&target).ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingLiveSubscription,
                format!("live view `{}` has no active subscription", view.name()),
            )
        })?;
        let current = state
            .async_result_state
            .as_ref()
            .map(WorthQueryRuntimeAsyncResultState::kind);
        let binding = state.async_source_binding.as_ref().ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
                format!("live view `{}` has no async binding", view.name()),
            )
        })?;
        if binding.declaration_identity_reference() != admitted.lowered().declaration_identity()
            || binding.current_request_identity_reference() != admitted.request_identity()
        {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::ForeignRequest,
                format!(
                    "live view `{}` is not bound to the denied declaration",
                    view.name()
                ),
            ));
        }
        if current == Some(terminal_kind) {
            return retained_terminal_batch(runtime_provenance, view, state);
        }
        if current != Some(WorthQueryRuntimeAsyncResultStateKind::Pending) {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::IllegalResultTransition,
                format!("live view `{}` is no longer pending", view.name()),
            ));
        }
        let basis = binding.current_basis_identity();
        let checkpoint = binding.current_generation_identity();
        let binding_identity = binding.binding_identity().clone();
        let causality =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    evidence_family,
                )
                .field_shape(WorthQueryEvidenceTag::new("live_target"), view.name())
                .field_shape(
                    WorthQueryEvidenceTag::new(request_field),
                    admitted.request_identity_for_reporting(),
                )
                .seal();
        let terminal =
            WorthQueryRuntimeAsyncResultState::new(terminal_kind, &causality, &basis, &checkpoint);
        state.async_result_state = Some(terminal.clone());
        let remask_posture = state.remask_posture.clone();
        Ok(WorthQueryAsyncResultTransitionBatch::admitted(
            runtime_provenance,
            view.name(),
            binding_identity,
            basis,
            checkpoint,
            remask_posture,
            vec![terminal],
            0,
        ))
    }
}

fn foreign_request(detail: impl Into<std::sync::Arc<str>>) -> WorthQueryAsyncSourceBindingError {
    WorthQueryAsyncSourceBindingError::new(
        WorthQueryAsyncSourceBindingErrorKind::ForeignRequest,
        detail,
    )
}

fn retained_terminal_batch<T>(
    runtime_provenance: super::WorthQueryRuntimeProvenance,
    view: &WorthQueryLiveView<T>,
    state: &super::WorthQueryRuntimeLiveSubscriptionState,
) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
    let binding = state.async_source_binding.as_ref().ok_or_else(|| {
        WorthQueryAsyncSourceBindingError::new(
            WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
            format!("live view `{}` has no async binding", view.name()),
        )
    })?;
    let retained = state.async_result_state.clone().ok_or_else(|| {
        WorthQueryAsyncSourceBindingError::new(
            WorthQueryAsyncSourceBindingErrorKind::IllegalResultTransition,
            format!("live view `{}` has no retained terminal state", view.name()),
        )
    })?;
    Ok(WorthQueryAsyncResultTransitionBatch::admitted(
        runtime_provenance,
        view.name(),
        binding.binding_identity().clone(),
        binding.current_basis_identity(),
        binding.current_generation_identity(),
        state.remask_posture.clone(),
        vec![retained],
        0,
    ))
}
