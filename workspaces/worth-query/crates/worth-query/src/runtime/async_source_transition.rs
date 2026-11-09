use worth_runtime_bridge::facade::BridgeMixedCauseOrdering;

use super::async_source_transition_plan::plan_async_result_transitions;
use super::{
    WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError,
    WorthQueryAsyncSourceBindingErrorKind, WorthQueryLiveArtifactTarget, WorthQueryLiveView,
    WorthQueryRuntime, WorthQueryRuntimeAsyncResultState,
};

impl WorthQueryRuntime {
    pub fn take_bridge_async_initial_result<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        let target = WorthQueryLiveArtifactTarget::from_view_name(view.name());
        let runtime_provenance = self.runtime_provenance();
        let state = self.live_subscriptions.get_mut(&target).ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingLiveSubscription,
                format!("live view `{}` has no active subscription", view.name()),
            )
        })?;
        let initial = state.async_result_state.clone().ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
                format!("live view `{}` has no async result state", view.name()),
            )
        })?;
        if initial.kind() != super::WorthQueryRuntimeAsyncResultStateKind::Pending {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::InitialStateNoLongerPending,
                format!(
                    "live view `{}` already advanced beyond Pending",
                    view.name()
                ),
            ));
        }
        let binding = state.async_source_binding.as_mut().ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
                format!("live view `{}` has no async source binding", view.name()),
            )
        })?;
        if !binding.take_initial_state_delivery() {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::InitialStateAlreadyDelivered,
                format!(
                    "live view `{}` already delivered its initial state",
                    view.name()
                ),
            ));
        }
        let expected_basis_identity = binding.current_basis_identity()?;
        let expected_checkpoint_identity = binding.current_generation_identity()?;
        let remask_posture = state.remask_posture.clone();
        Ok(WorthQueryAsyncResultTransitionBatch::admitted(
            runtime_provenance,
            view.name(),
            binding.binding_identity().clone(),
            expected_basis_identity,
            expected_checkpoint_identity,
            remask_posture,
            vec![initial],
            0,
        ))
    }

    pub fn admit_bridge_async_result_transitions<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        ordering: &BridgeMixedCauseOrdering,
    ) -> Result<WorthQueryAsyncResultTransitionBatch, WorthQueryAsyncSourceBindingError> {
        let target = WorthQueryLiveArtifactTarget::from_view_name(view.name());
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingLiveSubscription,
                format!("live view `{}` has no active subscription", view.name()),
            )
        })?;
        let binding = state.async_source_binding.clone().ok_or_else(|| {
            WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::MissingBinding,
                format!("live view `{}` has no async source binding", view.name()),
            )
        })?;
        let prior_kind = state
            .async_result_state
            .as_ref()
            .map(WorthQueryRuntimeAsyncResultState::kind);
        let plan = plan_async_result_transitions(view.name(), binding, prior_kind, ordering)?;
        let (binding, transitions, suppressed_duplicate_count) = plan.into_parts();
        let expected_basis = binding.current_basis_identity()?;
        let expected_checkpoint = binding.current_generation_identity()?;
        let remask_posture = state.remask_posture.clone();
        let mut states = Vec::with_capacity(transitions.len());
        for transition in transitions {
            let (projection, basis, checkpoint) = transition.into_parts();
            states.push(WorthQueryRuntimeAsyncResultState::new(
                projection.kind(),
                projection.causality_identity(),
                &basis,
                &checkpoint,
            ));
        }
        let binding_identity = binding.binding_identity().clone();
        let state = self
            .live_subscriptions
            .get_mut(&target)
            .expect("validated live target remains installed");
        if let Some(final_state) = states.last().cloned() {
            state.async_result_state = Some(final_state);
        }
        state.async_source_binding = Some(binding);
        Ok(WorthQueryAsyncResultTransitionBatch::admitted(
            self.runtime_provenance(),
            view.name(),
            binding_identity,
            expected_basis,
            expected_checkpoint,
            remask_posture,
            states,
            suppressed_duplicate_count,
        ))
    }
}
