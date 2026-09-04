use std::collections::VecDeque;

use crate::facade::host::{
    UiHostAdapterSessionAuthority, UiHostSessionReleaseOutcome, UiHostSessionReleaseReceipt,
    WorthUiOperationalHostAdapter,
};
use worth_ui_host_contract::WorthUiHostContract;

use super::*;

mod presentation;

impl WorthUiOperationalHostAdapter for ScriptedPresentationHost {
    fn operational_host_contract(&self) -> WorthUiHostContract {
        self.state.lock().unwrap().contract
    }

    fn operational_capability_report(&self) -> WorthUiHostCapabilityReport {
        self.state.lock().unwrap().capabilities.clone()
    }

    fn operational_protocol_contract(&self) -> worth_ui_host_contract::UiHostProtocolContract {
        self.state.lock().unwrap().protocol
    }

    fn open_host_session(
        &self,
        authority: &UiHostAdapterSessionAuthority,
    ) -> Result<(), worth_ui_host_contract::UiHostObservationSessionRegistrationDenial> {
        self.observation_retention
            .register_session(authority.host_session_identity())
    }

    fn drain_host_observations(
        &self,
        authority: &UiHostAdapterSessionAuthority,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationDrain,
        worth_ui_host_contract::UiHostObservationDrainDenial,
    > {
        Ok(self
            .observation_retention
            .drain(authority.host_session_identity()))
    }

    fn install_input_recipient(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        if binding.host_session() != authority.host_session_identity() {
            return false;
        }
        self.state.lock().unwrap().input_recipient = Some(binding);
        true
    }

    fn clear_input_recipient(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        if binding.host_session() != authority.host_session_identity() {
            return false;
        }
        let mut state = self.state.lock().unwrap();
        if state.input_recipient != Some(binding) {
            return false;
        }
        state.input_recipient = None;
        true
    }

    fn measurement_environment_report(
        &self,
    ) -> worth_ui_host_contract::UiHostMeasurementEnvironmentReport {
        let state = self.state.lock().unwrap();
        let supports = |capability| state.capabilities.supports(capability);
        worth_ui_host_contract::UiHostMeasurementEnvironmentReport::new(
            supports(worth_ui_host_contract::WorthUiHostCapability::ViewportObservation)
                .then_some(state.viewport_environment_generation),
            supports(worth_ui_host_contract::WorthUiHostCapability::DpiObservation)
                .then_some(state.viewport_environment_generation),
            (supports(worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement)
                || supports(
                    worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
                )
                || supports(worth_ui_host_contract::WorthUiHostCapability::FontMetrics))
            .then_some(state.font_environment_generation),
            (supports(
                worth_ui_host_contract::WorthUiHostCapability::NativeControlIntrinsicMeasurement,
            ) || supports(worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation)
                || supports(
                    worth_ui_host_contract::WorthUiHostCapability::ScrollContainerObservation,
                ))
            .then_some(state.adapter_environment_generation),
        )
    }

    fn visual_capture_capability(&self) -> worth_ui_host_contract::UiHostCaptureCapability {
        self.state.lock().unwrap().visual_capture_capability
    }

    fn capture_visual_presentation(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureObservationOutcome {
        if !authority.admits_visual_capture(request) {
            return worth_ui_host_contract::UiHostCaptureObservationOutcome::Unsupported;
        }
        let mut state = self.state.lock().unwrap();
        state.visual_capture_calls.push(request);
        let script = state
            .visual_captures
            .pop_front()
            .expect("script names every visual capture outcome");
        visual_capture_script::observe(request, script)
    }

    fn cancel_visual_capture(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureCancellationOutcome {
        if !authority.admits_visual_capture(request) {
            return worth_ui_host_contract::UiHostCaptureCancellationOutcome::CleanupIndeterminate;
        }
        let mut state = self.state.lock().unwrap();
        state.visual_cancellation_calls.push(request);
        state.visual_cancellation_outcome
    }

    fn register_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceRegistrationOutcome {
        if request.host_session_identity() != authority.host_session_identity() {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        let mut state = self.state.lock().unwrap();
        if state
            .registrations
            .contains_key(&request.host_surface_identity())
        {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        state
            .registrations
            .insert(request.host_surface_identity(), request);
        if std::mem::take(&mut state.indeterminate_next_registration) {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RegistrationIndeterminate(
                worth_ui_host_contract::UiHostSurfaceRegistrationIndeterminate::after_effects_may_have_begun(request),
            );
        }
        worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RegisteredKnownEmpty
    }

    fn deregister_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome {
        if request.host_session_identity() != authority.host_session_identity() {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        let mut state = self.state.lock().unwrap();
        if state.registrations.remove(&request.host_surface_identity()) != Some(request) {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        if std::mem::take(&mut state.wrong_next_deregistration_receipt) {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::Deregistered(
                worth_ui_host_contract::UiHostSurfaceDeregistrationReceipt::from_runtime(
                    request.host_session_identity().wrapping_add(1),
                    request.host_surface_identity(),
                ),
            );
        }
        worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::Deregistered(
            worth_ui_host_contract::UiHostSurfaceDeregistrationReceipt::from_runtime(
                request.host_session_identity(),
                request.host_surface_identity(),
            ),
        )
    }

    fn place_semantic_focus(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: worth_ui_host_contract::UiHostFocusPlacementRequest,
    ) -> worth_ui_host_contract::UiHostFocusPlacementAcknowledgement {
        let disposition = if request.host_session() != authority.host_session_identity() {
            worth_ui_host_contract::UiHostFocusPlacementDisposition::RejectedBeforeEffect(
                worth_ui_host_contract::UiHostFocusPlacementRejection::ForeignSurface,
            )
        } else {
            self.state.lock().unwrap().last_focus_placement = Some(request);
            worth_ui_host_contract::UiHostFocusPlacementDisposition::Applied
        };
        worth_ui_host_contract::UiHostFocusPlacementAcknowledgement::settled(request, disposition)
    }

    fn present_mounted_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> UiHostSurfacePresentationOutcome {
        self.present_scripted_mounted_surface(authority, request)
    }

    fn complete_mounted_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
    ) -> UiHostSurfaceInFlightCompletion {
        if !authority.admits_mounted_completion_token(&token) {
            return UiHostSurfaceInFlightCompletion::PresentationIndeterminate;
        }
        let identity = token.diagnostic_value();
        let mut state = self.state.lock().unwrap();
        let completion = state
            .completions
            .get_mut(&identity)
            .and_then(VecDeque::pop_front);
        match completion {
            Some(ScriptedSurfaceCompletion::Pending) => {
                UiHostSurfaceInFlightCompletion::Pending(token)
            }
            Some(ScriptedSurfaceCompletion::RejectedBeforeEffects(denial)) => {
                clear_token_state(&mut state, identity);
                UiHostSurfaceInFlightCompletion::RejectedBeforeEffects(denial)
            }
            Some(ScriptedSurfaceCompletion::Presented(completion)) => {
                clear_token_state(&mut state, identity);
                UiHostSurfaceInFlightCompletion::Presented(completion)
            }
            Some(ScriptedSurfaceCompletion::Superseded(observation)) => {
                clear_token_state(&mut state, identity);
                UiHostSurfaceInFlightCompletion::Superseded(observation)
            }
            Some(ScriptedSurfaceCompletion::PresentationIndeterminate) | None => {
                clear_token_state(&mut state, identity);
                UiHostSurfaceInFlightCompletion::PresentationIndeterminate
            }
        }
    }

    fn cancel_mounted_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
        _reason: worth_ui_host_contract::UiHostSurfaceStopReason,
    ) -> UiHostSurfaceCancellationOutcome {
        if !authority.admits_mounted_completion_token(&token) {
            return UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun;
        }
        let identity = token.diagnostic_value();
        let mut state = self.state.lock().unwrap();
        state.cancellation_calls.push(identity);
        state.completions.remove(&identity);
        state.token_sessions.remove(&identity);
        state
            .cancellations
            .remove(&identity)
            .unwrap_or(UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun)
    }

    fn release_host_session(
        &self,
        authority: &UiHostAdapterSessionAuthority,
    ) -> UiHostSessionReleaseOutcome {
        let mut state = self.state.lock().unwrap();
        let before = state.registrations.len();
        state.registrations.retain(|_, request| {
            request.host_session_identity() != authority.host_session_identity()
        });
        let session_tokens = state
            .token_sessions
            .iter()
            .filter_map(|(token, session)| {
                (*session == authority.host_session_identity()).then_some(*token)
            })
            .collect::<Vec<_>>();
        for token in session_tokens {
            clear_token_state(&mut state, token);
        }
        self.observation_retention
            .release_session(authority.host_session_identity());
        UiHostSessionReleaseOutcome::Released(UiHostSessionReleaseReceipt::released(
            authority.host_session_identity(),
            before - state.registrations.len(),
        ))
    }
}

fn clear_token_state(state: &mut ScriptedPresentationState, identity: u64) {
    state.completions.remove(&identity);
    state.cancellations.remove(&identity);
    state.token_sessions.remove(&identity);
}
