use crate::native::{
    UiNativeEffectPosture, UiNativeLifecycleProtocol, UiNativeRecoveryRegistry,
    UiNativeShutdownPhase,
};

mod input;
mod presentation;
#[cfg(feature = "certification-support")]
mod protocol_execution;
mod recovery;
#[cfg(feature = "certification-support")]
mod shutdown;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeSurfaceBasisTransition {
    ZeroSized,
    Minimized,
    Resize,
    Dpi,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeLifecycleDirective {
    #[cfg(feature = "certification-support")]
    RetryAfterTimeout,
    WaitForVisibility,
    #[cfg(feature = "certification-support")]
    RejectValidation,
    /// The retained device, pipelines and draw lists carry onto a successor
    /// target of the same scale; the next presentation repaints it whole.
    Repaint,
    Reconstruct(crate::native::presentation::UiNativePresentationRecoveryClass),
}

#[cfg(feature = "certification-support")]
pub(super) fn run_protocol(
    schedule: super::protocol_world::UiNativeLifecycleProtocolSchedule,
) -> super::protocol_world::UiNativeLifecycleProtocolReport {
    protocol_execution::UiProtocolExecution::new(schedule).run()
}

/// Ordinary lifecycle authority shared by the real host and contractual worlds.
pub(crate) struct UiNativeLifecycleOrchestrator {
    protocol: UiNativeLifecycleProtocol,
    recovery: UiNativeRecoveryRegistry,
    effect_posture: UiNativeEffectPosture,
    presentation_retry: super::presentation_retry::UiNativePresentationRetryPolicy,
    shutdown_phase: UiNativeShutdownPhase,
}

impl UiNativeLifecycleOrchestrator {
    pub(crate) fn new() -> Self {
        Self {
            protocol: UiNativeLifecycleProtocol::new(),
            recovery: UiNativeRecoveryRegistry::default(),
            effect_posture: UiNativeEffectPosture::BeforeEffects,
            presentation_retry: super::presentation_retry::UiNativePresentationRetryPolicy::new(),
            shutdown_phase: UiNativeShutdownPhase::Open,
        }
    }

    pub(crate) const fn classify_surface_failure(
        failure: crate::native::presentation::UiNativeSurfaceAcquireFailure,
    ) -> crate::native::presentation::UiNativeSurfaceFailureDisposition {
        crate::native::presentation::classify_surface_failure(failure)
    }

    pub(crate) const fn shutdown_phase(&self) -> UiNativeShutdownPhase {
        self.shutdown_phase
    }

    pub(crate) fn record_shutdown_phase(&mut self, phase: UiNativeShutdownPhase) {
        self.shutdown_phase = phase;
    }

    pub(crate) fn observe_presentation_retry_outcome(
        &mut self,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        outcome: &worth_ui_host_contract::UiHostSurfacePresentationOutcome,
    ) {
        self.presentation_retry.observe_outcome(attempt, outcome);
    }

    pub(crate) fn finalize_presentation_retry_round(
        &mut self,
        now: std::time::Instant,
    ) -> super::presentation_retry::UiNativePresentationRetryFinalization {
        self.presentation_retry.finalize_round(now)
    }

    pub(crate) const fn presentation_retry_wake(
        &self,
    ) -> Option<super::presentation_retry::UiNativePresentationRetryWake> {
        self.presentation_retry.wake()
    }

    pub(crate) fn consume_due_presentation_timeout(&mut self, now: std::time::Instant) -> bool {
        self.presentation_retry.consume_due_timeout(now)
    }

    pub(crate) fn consume_presentation_visibility(&mut self) -> bool {
        self.presentation_retry.consume_visibility()
    }

    pub(crate) fn clear_presentation_retry(&mut self) {
        self.presentation_retry.clear();
    }

    pub(crate) fn presentation_readiness_allowed(&self) -> bool {
        !matches!(
            self.protocol.phase(),
            crate::native::UiNativeLifecyclePhase::Closing
                | crate::native::UiNativeLifecyclePhase::Closed
        )
    }
}

impl Default for UiNativeLifecycleOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
