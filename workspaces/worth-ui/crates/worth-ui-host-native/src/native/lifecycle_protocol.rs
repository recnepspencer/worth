#[cfg(any(test, feature = "certification-support"))]
use winit::dpi::PhysicalPosition;
#[cfg(feature = "certification-support")]
use winit::event::ElementState;
use winit::event::WindowEvent;
#[cfg(feature = "certification-support")]
use winit::keyboard::{Key, PhysicalKey};
use worth_ui_host_contract::{
    UiHostInputRecipientBindingReceipt, UiHostObservationDrain,
    UiHostObservationSessionRegistrationDenial, UiHostPresentationEpoch,
};

use super::{
    UiNativeInputObservationDisposition, UiNativeInputObservationReport,
    UiNativeInputObservationState, UiNativePointerPositionWitness,
};

mod phase;
mod presentation;
mod transition;

pub use phase::UiNativeLifecyclePhase;
pub use transition::{
    UiNativeLifecycleEffect, UiNativeLifecycleRequiredAction, UiNativeLifecycleTransition,
};

pub struct UiNativeLifecycleProtocol {
    input: UiNativeInputObservationState,
    phase: UiNativeLifecyclePhase,
    predecessor: Option<UiHostPresentationEpoch>,
}

impl UiNativeLifecycleProtocol {
    pub fn new() -> Self {
        Self {
            input: UiNativeInputObservationState::new(),
            phase: UiNativeLifecyclePhase::BeforeFirstPresentation,
            predecessor: None,
        }
    }

    pub const fn phase(&self) -> UiNativeLifecyclePhase {
        self.phase
    }

    #[cfg(feature = "certification-support")]
    pub const fn predecessor_epoch(&self) -> Option<UiHostPresentationEpoch> {
        self.predecessor
    }

    pub fn install_initial_profile(&mut self, scale_factor: f64, physical_size: [u32; 2]) {
        self.input
            .install_initial_profile(scale_factor, physical_size);
    }

    pub fn observe_profile_transition_at(
        &mut self,
        scale_factor: f64,
        physical_size: [u32; 2],
        event_tick: u64,
    ) -> UiNativeLifecycleTransition {
        if matches!(
            self.phase,
            UiNativeLifecyclePhase::Closing | UiNativeLifecyclePhase::Closed
        ) {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        self.input
            .observe_profile_transition_at(scale_factor, physical_size, event_tick);
        if let Some(epoch) = self.completed_epoch() {
            self.phase = UiNativeLifecyclePhase::ProfileTransition;
            self.predecessor = Some(epoch);
        }
        self.transition(
            UiNativeLifecycleEffect::NoOp,
            0,
            Some(UiNativeLifecycleRequiredAction::EmitProfileEvidence),
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub fn observe_window_event_at(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
        pointer_position: Option<PhysicalPosition<f64>>,
    ) -> UiNativeLifecycleTransition {
        self.observe_window_event_at_with_pointer_witness(
            event,
            event_tick,
            pointer_position
                .map(UiNativePointerPositionWitness::EventTime)
                .unwrap_or(UiNativePointerPositionWitness::Unavailable),
        )
    }

    pub(crate) fn observe_window_event_at_with_pointer_witness(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
        pointer_witness: UiNativePointerPositionWitness,
    ) -> UiNativeLifecycleTransition {
        if self.input_is_suppressed() {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        let before_events = self.retained_event_count();
        let before_stops = self.input.report().stops().len();
        let disposition = self.input.observe_window_event_at_with_pointer_witness(
            event,
            event_tick,
            pointer_witness,
        );
        self.input_transition(disposition, before_events, before_stops)
    }

    #[cfg(feature = "certification-support")]
    pub fn observe_keyboard_components_at(
        &mut self,
        logical_key: &Key,
        physical_key: PhysicalKey,
        key_state: ElementState,
        repeat: bool,
        text: Option<&str>,
        event_tick: u64,
    ) -> UiNativeLifecycleTransition {
        if self.input_is_suppressed() {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        let before_events = self.retained_event_count();
        let before_stops = self.input.report().stops().len();
        let disposition = self.input.observe_keyboard_components_at(
            logical_key,
            physical_key,
            key_state,
            repeat,
            text,
            event_tick,
        );
        self.input_transition(disposition, before_events, before_stops)
    }

    pub fn request_close(&mut self) -> UiNativeLifecycleTransition {
        if self.phase == UiNativeLifecyclePhase::Closed {
            return self.transition(UiNativeLifecycleEffect::Closed, 0, None);
        }
        if self.input.has_retained_observations() {
            self.phase = UiNativeLifecyclePhase::Closing;
            self.predecessor = None;
            return self.transition(
                UiNativeLifecycleEffect::CloseDeferred,
                0,
                Some(UiNativeLifecycleRequiredAction::DrainRetained),
            );
        }
        self.close();
        self.transition(UiNativeLifecycleEffect::Closed, 0, None)
    }

    pub fn drain(&mut self, host_session: u64) -> UiHostObservationDrain {
        self.input.drain(host_session)
    }

    pub fn register_session(
        &self,
        host_session: u64,
    ) -> Result<(), UiHostObservationSessionRegistrationDenial> {
        self.input.register_session(host_session)
    }

    pub fn install_input_recipient(&mut self, binding: UiHostInputRecipientBindingReceipt) -> bool {
        self.input.install_input_recipient(binding)
    }

    pub fn clear_input_recipient(&mut self, binding: UiHostInputRecipientBindingReceipt) -> bool {
        self.input.clear_input_recipient(binding)
    }

    pub fn has_retained_observations(&self) -> bool {
        self.input.has_retained_observations()
    }

    pub(crate) fn ime_composition_posture(&self) -> super::UiNativeImeCompositionPosture {
        self.input.ime_composition_posture()
    }

    pub(crate) fn observation_drain_capacity_reached(&self) -> bool {
        self.input.observation_drain_capacity_reached()
    }

    pub fn release_session(&mut self, host_session: u64) {
        self.input.release_session(host_session);
        if self.input.report().last_completed_presentation().is_none() {
            self.phase = UiNativeLifecyclePhase::BeforeFirstPresentation;
            self.predecessor = None;
        }
    }

    pub fn close(&mut self) {
        self.input.close();
        self.phase = UiNativeLifecyclePhase::Closed;
        self.predecessor = None;
    }

    pub fn report(&self) -> UiNativeInputObservationReport {
        self.input.report()
    }

    fn input_transition(
        &self,
        disposition: UiNativeInputObservationDisposition,
        before_events: u64,
        before_stops: usize,
    ) -> UiNativeLifecycleTransition {
        let report = self.input.report();
        let effect = match disposition {
            UiNativeInputObservationDisposition::Retained => UiNativeLifecycleEffect::Retained,
            UiNativeInputObservationDisposition::Stopped => UiNativeLifecycleEffect::Denied(
                report
                    .terminal_stop()
                    .expect("stopped input has a typed terminal reason"),
            ),
            UiNativeInputObservationDisposition::Ignored => {
                report.stops().get(before_stops).copied().map_or(
                    UiNativeLifecycleEffect::Ignored,
                    UiNativeLifecycleEffect::Denied,
                )
            }
        };
        self.transition(
            effect,
            report.retained_event_count().saturating_sub(before_events),
            None,
        )
    }

    fn input_is_suppressed(&self) -> bool {
        matches!(
            self.phase,
            UiNativeLifecyclePhase::Closing | UiNativeLifecyclePhase::Closed
        )
    }

    fn completed_epoch(&self) -> Option<UiHostPresentationEpoch> {
        self.input
            .report()
            .last_completed_presentation()
            .map(|basis| basis.epoch())
    }

    fn retained_event_count(&self) -> u64 {
        self.input.report().retained_event_count()
    }

    fn transition(
        &self,
        effect: UiNativeLifecycleEffect,
        retained_delta: u64,
        required_action: Option<UiNativeLifecycleRequiredAction>,
    ) -> UiNativeLifecycleTransition {
        UiNativeLifecycleTransition::new(
            self.phase,
            effect,
            retained_delta,
            self.predecessor,
            required_action,
        )
    }
}

impl Default for UiNativeLifecycleProtocol {
    fn default() -> Self {
        Self::new()
    }
}
