use std::cell::RefCell;
use std::rc::Rc;

use worth_ui_host_contract::{
    UiHostSessionReleaseOutcome, UiHostSessionReleaseReceipt, UiHostSurfacePresentationMode,
    UiHostSurfacePresentationOutcome, UiHostSurfaceRegistrationDenial,
    UiHostSurfaceRegistrationRequest, UiMountedFrameConsumptionView, WorthUiHostCapabilityReport,
    WorthUiHostContract, WorthUiHostMechanicsAdapter,
};

use super::UiNativeHostState;

mod capability;
#[cfg(feature = "certification-support")]
pub use capability::staged_appearance_report;
mod construction;
mod measurement;
mod presentation;
#[path = "mechanics_adapter/presentation/text_atlas.rs"]
mod presentation_text_atlas;
pub(super) mod text_atlas;

#[cfg(test)]
pub(crate) use text_atlas::seed_pending_atlas_for_event_loop;

pub(crate) struct WorthUiNativeMechanicsAdapter {
    state: Rc<RefCell<UiNativeHostState>>,
    profile: crate::UiNativePlatformProfileIdentity,
}

impl WorthUiHostMechanicsAdapter for WorthUiNativeMechanicsAdapter {
    fn mechanical_host_contract(&self) -> WorthUiHostContract {
        debug_assert_eq!(
            self.profile,
            crate::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V1
        );
        WorthUiHostContract::native()
    }

    fn mechanical_capability_report(&self) -> WorthUiHostCapabilityReport {
        capability::report()
    }

    fn mechanical_visual_capture_capability(
        &self,
    ) -> worth_ui_host_contract::UiHostCaptureCapability {
        super::capture::capability()
    }

    fn perform_visual_capture(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureObservationOutcome {
        super::capture::observe(&mut self.state.borrow_mut(), request)
    }

    fn perform_visual_capture_cancellation(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureCancellationOutcome {
        super::capture::cancel(&mut self.state.borrow_mut(), request)
    }

    fn perform_focus_placement(
        &self,
        request: worth_ui_host_contract::UiHostFocusPlacementRequest,
    ) -> worth_ui_host_contract::UiHostFocusPlacementAcknowledgement {
        super::solicited_effect::focus_placement::perform(&mut self.state.borrow_mut(), request)
    }

    fn drain_mechanical_host_observations(
        &self,
        host_session_identity: u64,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationDrain,
        worth_ui_host_contract::UiHostObservationDrainDenial,
    > {
        Ok(self
            .state
            .borrow_mut()
            .lifecycle
            .drain_observations(host_session_identity))
    }

    fn register_mechanical_host_session(
        &self,
        host_session_identity: u64,
    ) -> Result<(), worth_ui_host_contract::UiHostObservationSessionRegistrationDenial> {
        self.state
            .borrow()
            .lifecycle
            .register_session(host_session_identity)
    }

    fn install_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.state
            .borrow_mut()
            .lifecycle
            .install_input_recipient(binding)
    }

    fn clear_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.state
            .borrow_mut()
            .lifecycle
            .clear_input_recipient(binding)
    }

    fn perform_surface_registration(
        &self,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceRegistrationOutcome {
        let key = request.binding_generation().diagnostic_value();
        let mut state = self.state.borrow_mut();
        if state
            .registrations
            .get(&key)
            .is_some_and(|current| *current != request)
        {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        if !state.registrations.contains_key(&key) {
            let owner = match state
                .resources
                .register(super::UiNativeResourceClass::HostRegistration)
            {
                Ok(owner) => owner,
                Err(()) => {
                    return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                        UiHostSurfaceRegistrationDenial::CapacityExceeded,
                    );
                }
            };
            state.lifecycle.claim_recovery(
                super::UiNativeRecoveryLineage::from_registration(request),
                key,
            );
            let recovery_predecessors = state
                .registrations
                .iter()
                .filter(|(predecessor, registered)| {
                    **predecessor != key
                        && registered.host_session_identity() == request.host_session_identity()
                        && registered.semantic_surface_identity()
                            == request.semantic_surface_identity()
                        && registered.host_surface_identity() == request.host_surface_identity()
                        && registered.presentation_mode() == request.presentation_mode()
                        && state.lifecycle.recovery_required(**predecessor)
                })
                .map(|(predecessor, _)| *predecessor)
                .collect::<Vec<_>>();
            for predecessor in recovery_predecessors {
                state.lifecycle.transfer_recovery(predecessor, key);
            }
            state.registrations.insert(key, request);
            state.registration_resources.insert(key, owner);
        }
        worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RegisteredKnownEmpty
    }

    fn perform_mounted_surface_presentation(
        &self,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> UiHostSurfacePresentationOutcome {
        if view.requirement().presentation_mode() != UiHostSurfacePresentationMode::NativeDisplay {
            return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfacePresentationDenial::UnsupportedPresentationMode(
                    view.requirement().presentation_mode(),
                ),
            );
        }
        presentation::perform_native_presentation(&mut self.state.borrow_mut(), view)
    }

    fn perform_mounted_surface_completion(
        &self,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
    ) -> worth_ui_host_contract::UiHostSurfaceInFlightCompletion {
        let mut state = self.state.borrow_mut();
        if presentation::owns_completion(&state, &token) {
            presentation::complete_pending(&mut state, token)
        } else {
            presentation_text_atlas::complete(&mut state, token)
        }
    }

    fn perform_mounted_surface_cancellation(
        &self,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
        reason: worth_ui_host_contract::UiHostSurfaceStopReason,
    ) -> worth_ui_host_contract::UiHostSurfaceCancellationOutcome {
        let mut state = self.state.borrow_mut();
        if presentation::owns_completion(&state, &token) {
            presentation::stop_pending(&mut state, token)
        } else {
            presentation_text_atlas::stop(&mut state, token, reason)
        }
    }

    fn perform_surface_deregistration(
        &self,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome {
        let key = request.binding_generation().diagnostic_value();
        let mut state = self.state.borrow_mut();
        if state.registrations.get(&key) != Some(&request) {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        let binding_pins = state.text_pins_by_binding.remove(&key).unwrap_or_default();
        let live_pins = state.text_atlas.pin_observations();
        let releases = binding_pins
            .iter()
            .copied()
            .filter(|pin| {
                !state
                    .text_pins_by_binding
                    .iter()
                    .any(|(retained_binding, retained)| {
                        state.registrations.contains_key(retained_binding) && retained.contains(pin)
                    })
            })
            .filter(|pin| live_pins.iter().any(|live| live.matches(*pin)))
            .collect::<Vec<_>>();
        if !releases.is_empty() {
            let Ok(attempt) =
                worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
            else {
                state.text_pins_by_binding.insert(key, binding_pins);
                return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::DeregistrationIndeterminate(
                    worth_ui_host_contract::UiHostSurfaceDeregistrationIndeterminate::after_effects_may_have_begun(request),
                );
            };
            let empty = [];
            let outcome = text_atlas::release_pins(
                &mut state,
                worth_ui_host_contract::UiMountedTextPinReleaseRequest::from_runtime(
                    request, attempt,
                ),
                worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(
                    &empty, &releases,
                ),
            );
            if !matches!(
                outcome,
                worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Committed(_)
            ) {
                state.text_pins_by_binding.insert(key, binding_pins);
                return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::DeregistrationIndeterminate(
                    worth_ui_host_contract::UiHostSurfaceDeregistrationIndeterminate::after_effects_may_have_begun(request),
                );
            }
        }
        state.lifecycle.park_recovery(
            key,
            super::UiNativeRecoveryLineage::from_registration(request),
        );
        state.registrations.remove(&key);
        state.captures.remove_binding(request.binding_generation());
        state.retained_draw_lists.remove(&key);
        let Some(owner) = state.registration_resources.remove(&key) else {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::DeregistrationIndeterminate(
                worth_ui_host_contract::UiHostSurfaceDeregistrationIndeterminate::after_effects_may_have_begun(request),
            );
        };
        state
            .resources
            .release(owner)
            .expect("registration owner must remain exact");
        worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::Deregistered(
            worth_ui_host_contract::UiHostSurfaceDeregistrationReceipt::from_runtime(
                request.host_session_identity(),
                request.host_surface_identity(),
            ),
        )
    }

    fn release_mechanical_host_session(
        &self,
        host_session_identity: u64,
    ) -> UiHostSessionReleaseOutcome {
        let mut state = self.state.borrow_mut();
        state.semantic_focus.remove(&host_session_identity);
        state.lifecycle.release_session(host_session_identity);
        let pending_tokens = state
            .pending_text_presentations
            .iter()
            .filter_map(|(token, pending)| {
                (pending.atlas.host_session() == host_session_identity).then_some(*token)
            })
            .collect::<Vec<_>>();
        if !pending_tokens.is_empty() {
            for token in pending_tokens {
                if let Some(pending) = state.pending_text_presentations.remove(&token) {
                    let _ = state.cancel_pending_text_atlas(pending.atlas);
                }
            }
            return UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                worth_ui_host_contract::UiHostSessionReleaseIndeterminate::after_effects_may_have_begun(
                    host_session_identity,
                ),
            );
        }
        let before = state.registrations.len();
        let released = state
            .registrations
            .iter()
            .filter_map(|(key, request)| {
                (request.host_session_identity() == host_session_identity).then_some(*key)
            })
            .collect::<Vec<_>>();
        let retained_pins = state
            .text_pins_by_binding
            .iter()
            .filter(|(key, _)| !released.contains(key))
            .flat_map(|(_, pins)| pins.iter().copied())
            .collect::<Vec<_>>();
        let released_pins = released
            .iter()
            .filter_map(|key| state.text_pins_by_binding.get(key))
            .flat_map(|pins| pins.iter().copied())
            .filter(|pin| !retained_pins.contains(pin))
            .collect::<Vec<_>>();
        if !released_pins.is_empty() {
            let Some(request) = released
                .first()
                .and_then(|key| state.registrations.get(key))
                .copied()
            else {
                return UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                    worth_ui_host_contract::UiHostSessionReleaseIndeterminate::after_effects_may_have_begun(
                        host_session_identity,
                    ),
                );
            };
            let Ok(attempt) =
                worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
            else {
                return UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                    worth_ui_host_contract::UiHostSessionReleaseIndeterminate::after_effects_may_have_begun(
                        host_session_identity,
                    ),
                );
            };
            let empty = [];
            let outcome = text_atlas::release_pins(
                &mut state,
                worth_ui_host_contract::UiMountedTextPinReleaseRequest::from_runtime(
                    request, attempt,
                ),
                worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(
                    &empty,
                    &released_pins,
                ),
            );
            if !matches!(
                outcome,
                worth_ui_host_contract::UiGlyphRasterTransactionOutcome::Committed(_)
            ) {
                return UiHostSessionReleaseOutcome::ReleaseIndeterminate(
                    worth_ui_host_contract::UiHostSessionReleaseIndeterminate::after_effects_may_have_begun(
                        host_session_identity,
                    ),
                );
            }
        }
        for key in released {
            if let Some(request) = state.registrations.get(&key).copied() {
                state.captures.remove_binding(request.binding_generation());
            }
            state.registrations.remove(&key);
            state.retained_draw_lists.remove(&key);
            state.lifecycle.resolve_recovery(key);
            state.text_pins_by_binding.remove(&key);
            let owner = state
                .registration_resources
                .remove(&key)
                .expect("registration resource must exist");
            state
                .resources
                .release(owner)
                .expect("registration owner must remain exact");
        }
        UiHostSessionReleaseOutcome::Released(UiHostSessionReleaseReceipt::released(
            host_session_identity,
            before - state.registrations.len(),
        ))
    }
}
