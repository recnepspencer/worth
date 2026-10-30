use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostProtocolAgreement,
    UiHostSurfaceIdentity, UiMountedFrameIdentity, UiSurfaceBindingGeneration,
};

use super::{
    UiNativeLifecycleEffect, UiNativeLifecyclePhase, UiNativeLifecycleProtocol,
    UiNativeLifecycleRequiredAction, UiNativeLifecycleTransition,
};

impl UiNativeLifecycleProtocol {
    pub fn record_completed_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        presentation: UiHostObservationPresentationBasis,
    ) -> UiNativeLifecycleTransition {
        if matches!(
            self.phase,
            UiNativeLifecyclePhase::Closing | UiNativeLifecyclePhase::Closed
        ) {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        let before = self.retained_event_count();
        let completed =
            self.input
                .record_completed_presentation(protocol, host_session, presentation);
        if completed {
            self.phase = UiNativeLifecyclePhase::Presented;
            self.predecessor = None;
        }
        self.transition(
            if completed {
                UiNativeLifecycleEffect::PresentationCompleted
            } else {
                self.latest_presentation_denial()
            },
            self.retained_event_count().saturating_sub(before),
            None,
        )
    }

    pub fn remember_pending_presentation(
        &mut self,
        protocol: UiHostProtocolAgreement,
        host_session: u64,
        host_surface: UiHostSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        completion_identity: u64,
    ) -> UiNativeLifecycleTransition {
        if matches!(
            self.phase,
            UiNativeLifecyclePhase::Closing | UiNativeLifecyclePhase::Closed
        ) {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        let remembered = self.input.remember_pending_presentation(
            protocol,
            host_session,
            host_surface,
            binding,
            completion_identity,
        );
        if remembered && let Some(epoch) = self.completed_epoch() {
            self.phase = UiNativeLifecyclePhase::SuccessorInFlight;
            self.predecessor = Some(epoch);
        }
        self.transition(
            if remembered {
                UiNativeLifecycleEffect::NoOp
            } else {
                self.latest_presentation_denial()
            },
            0,
            remembered.then_some(UiNativeLifecycleRequiredAction::CompletePresentation),
        )
    }

    pub fn complete_pending_presentation(
        &mut self,
        frame: UiMountedFrameIdentity,
        binding: UiSurfaceBindingGeneration,
        epoch: UiHostPresentationEpoch,
        completion_identity: u64,
    ) -> UiNativeLifecycleTransition {
        if matches!(
            self.phase,
            UiNativeLifecyclePhase::Closing | UiNativeLifecyclePhase::Closed
        ) {
            return self.transition(UiNativeLifecycleEffect::NoOp, 0, None);
        }
        let before = self.retained_event_count();
        let completed =
            self.input
                .complete_pending_presentation(frame, binding, epoch, completion_identity);
        if completed {
            self.phase = UiNativeLifecyclePhase::Presented;
            self.predecessor = None;
        }
        self.transition(
            if completed {
                UiNativeLifecycleEffect::PresentationCompleted
            } else {
                self.latest_presentation_denial()
            },
            self.retained_event_count().saturating_sub(before),
            None,
        )
    }

    pub fn abandon_pending_presentation(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        completion_identity: Option<u64>,
    ) {
        self.input
            .abandon_pending_presentation(binding, completion_identity);
        if self.phase == UiNativeLifecyclePhase::SuccessorInFlight
            && !self.input.has_pending_presentations()
        {
            self.phase = if self.completed_epoch().is_some() {
                UiNativeLifecyclePhase::Presented
            } else {
                UiNativeLifecyclePhase::BeforeFirstPresentation
            };
            self.predecessor = None;
        }
    }

    fn latest_presentation_denial(&self) -> UiNativeLifecycleEffect {
        self.input.report().terminal_stop().map_or(
            UiNativeLifecycleEffect::NoOp,
            UiNativeLifecycleEffect::Denied,
        )
    }
}
