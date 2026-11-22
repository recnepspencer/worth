//! Transport-fault classification without guessing external completion (R8.24).

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;

use super::causal_event::ExternalEffectPosture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalRailTransportFault {
    Timeout,
    Disconnect,
    LostResponse,
    DuplicatedAcknowledgement,
    UnsupportedProtocolVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    UnknownProviderOutcome,
    ObservationDerivationDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectClassification {
    fault: ExternalRailTransportFault,
    attempt: ExternalEffectPosture,
    decision_time: Option<WorthQueryRuntimeTimeSample>,
}

impl ExternalEffectClassification {
    pub fn from_attempt_posture(
        fault: ExternalRailTransportFault,
        attempt: ExternalEffectPosture,
        clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
    ) -> Self {
        let decision_time = clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
            .ok();
        Self {
            fault,
            attempt,
            decision_time,
        }
    }

    pub const fn fault(&self) -> ExternalRailTransportFault {
        self.fault
    }

    pub const fn attempt_posture(&self) -> &ExternalEffectPosture {
        &self.attempt
    }

    pub const fn decision_time(&self) -> Option<&WorthQueryRuntimeTimeSample> {
        self.decision_time.as_ref()
    }

    pub fn is_external_completion(&self) -> bool {
        false
    }
}
