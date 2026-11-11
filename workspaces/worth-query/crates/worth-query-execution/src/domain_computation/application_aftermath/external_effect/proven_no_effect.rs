//! Proven no effect: the external owner determined the attempt changed nothing.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;

use super::causal_event::{DispatchAttemptEvent, ExternalEffectPosture};

pub use worth_foundational::facade::ExternalEffectNoEffectCause;

/// The determinate outcome of one dispatch attempt whose external effect did
/// not happen. It is not unresolved (nothing remains unknown), not completion
/// (nothing was performed) and not compensation (nothing exists to undo).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectProvenNoEffect {
    cause: ExternalEffectNoEffectCause,
    attempt: ExternalEffectPosture,
    /// Exact trusted-time sample taken when the proof was observed.
    decision_time: Option<WorthQueryRuntimeTimeSample>,
}

impl ExternalEffectProvenNoEffect {
    pub const fn cause(&self) -> ExternalEffectNoEffectCause {
        self.cause
    }

    pub const fn attempt_posture(&self) -> &ExternalEffectPosture {
        &self.attempt
    }

    pub const fn decision_time(&self) -> Option<&WorthQueryRuntimeTimeSample> {
        self.decision_time.as_ref()
    }
}

/// Record the external owner's proof against the exact attempt it answered.
pub(super) fn prove_no_effect(
    cause: ExternalEffectNoEffectCause,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ExternalEffectProvenNoEffect {
    ExternalEffectProvenNoEffect {
        cause,
        attempt: attempt.attempt().clone(),
        decision_time: clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
            .ok(),
    }
}
