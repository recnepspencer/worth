#[cfg(test)]
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use worth_ui_host_contract::{
    UiHostInputRecipientBindingReceipt, UiHostObservationDrain, UiHostObservationPresentationBasis,
    UiHostObservationSessionRegistrationDenial, UiHostPresentationEpoch, UiHostProtocolAgreement,
    UiMountedFrameIdentity, UiSurfaceBindingGeneration,
};

use crate::native::{
    UiNativeInputObservationReport, UiNativeLifecycleTransition, UiNativePointerPositionWitness,
};

impl super::UiNativeLifecycleOrchestrator {
    pub(crate) fn begin_input_retention_recovery(
        &self,
    ) -> Option<crate::UiNativeInputRecoveryGrant> {
        self.protocol.begin_input_retention_recovery()
    }

    pub(crate) fn complete_input_retention_recovery(
        &mut self,
        acknowledgement: crate::UiNativeInputRecoveryAcknowledgement,
    ) -> bool {
        self.protocol
            .complete_input_retention_recovery(acknowledgement)
    }
    pub(crate) fn install_initial_profile(&mut self, scale_factor: f64, size: [u32; 2]) {
        self.protocol.install_initial_profile(scale_factor, size);
    }

    pub(crate) fn observe_profile_transition_at(
        &mut self,
        scale_factor: f64,
        size: [u32; 2],
        event_tick: u64,
    ) -> UiNativeLifecycleTransition {
        self.protocol
            .observe_profile_transition_at(scale_factor, size, event_tick)
    }

    #[cfg(test)]
    pub(crate) fn observe_window_event_at(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
        pointer: Option<PhysicalPosition<f64>>,
    ) -> UiNativeLifecycleTransition {
        self.protocol
            .observe_window_event_at(event, event_tick, pointer)
    }

    pub(crate) fn observe_window_event_at_with_pointer_witness(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
        pointer: UiNativePointerPositionWitness,
    ) -> UiNativeLifecycleTransition {
        self.protocol
            .observe_window_event_at_with_pointer_witness(event, event_tick, pointer)
    }

    pub(crate) fn request_close(&mut self) -> UiNativeLifecycleTransition {
        self.clear_presentation_retry();
        self.protocol.request_close()
    }

    pub(crate) fn close_input_protocol(&mut self) {
        self.protocol.close();
    }

    pub(crate) fn input_report(&self) -> UiNativeInputObservationReport {
        self.protocol.report()
    }

    pub(crate) fn has_retained_observations(&self) -> bool {
        self.protocol.has_retained_observations()
    }

    pub(crate) fn ime_composition_posture(&self) -> crate::native::UiNativeImeCompositionPosture {
        self.protocol.ime_composition_posture()
    }

    pub(crate) fn observation_drain_capacity_reached(&self) -> bool {
        self.protocol.observation_drain_capacity_reached()
    }

    pub(crate) fn drain_observations(&mut self, host_session: u64) -> UiHostObservationDrain {
        self.protocol.drain(host_session)
    }

    pub(crate) fn register_session(
        &self,
        host_session: u64,
    ) -> Result<(), UiHostObservationSessionRegistrationDenial> {
        self.protocol.register_session(host_session)
    }

    pub(crate) fn install_input_recipient(
        &mut self,
        binding: UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.protocol.install_input_recipient(binding)
    }

    pub(crate) fn clear_input_recipient(
        &mut self,
        binding: UiHostInputRecipientBindingReceipt,
    ) -> bool {
        self.protocol.clear_input_recipient(binding)
    }

    pub(crate) fn release_session(&mut self, host_session: u64) {
        self.protocol.release_session(host_session);
    }

    pub(crate) fn record_completed_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        presentation: UiHostObservationPresentationBasis,
    ) -> UiNativeLifecycleTransition {
        self.protocol
            .record_completed_presentation(protocol, host_session, presentation)
    }

    pub(crate) fn remember_pending_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        completion_identity: u64,
    ) -> UiNativeLifecycleTransition {
        self.protocol.remember_pending_presentation(
            protocol,
            host_session,
            host_surface,
            binding,
            completion_identity,
        )
    }

    pub(crate) fn complete_pending_presentation(
        &mut self,
        frame: UiMountedFrameIdentity,
        binding: UiSurfaceBindingGeneration,
        epoch: UiHostPresentationEpoch,
        completion_identity: u64,
    ) -> UiNativeLifecycleTransition {
        self.protocol
            .complete_pending_presentation(frame, binding, epoch, completion_identity)
    }

    pub(crate) fn abandon_pending_presentation(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        completion_identity: Option<u64>,
    ) {
        self.protocol
            .abandon_pending_presentation(binding, completion_identity);
    }
}
