//! Proven no effect: the external owner determined the attempt changed nothing.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;

use super::causal_event::ExternalEffectPosture;

pub use worth_foundational::facade::ExternalEffectNoEffectCause;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectProvenNoEffect {
    cause: ExternalEffectNoEffectCause,
    attempt: ExternalEffectPosture,
    decision_time: Option<WorthQueryRuntimeTimeSample>,
}

impl ExternalEffectProvenNoEffect {
    pub fn from_attempt_posture(
        cause: ExternalEffectNoEffectCause,
        attempt: ExternalEffectPosture,
        clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
    ) -> Self {
        Self {
            cause,
            attempt,
            decision_time: clock
                .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
                .ok(),
        }
    }

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
