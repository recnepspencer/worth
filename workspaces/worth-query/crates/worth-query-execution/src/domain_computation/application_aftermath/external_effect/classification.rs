//! Transport-fault classification without guessing external completion (R8.24).

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;

use super::causal_event::DispatchAttemptEvent;
use super::causal_event::ExternalEffectPosture;

/// Observed external-rail transport faults. None of these are completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalRailTransportFault {
    Timeout,
    Disconnect,
    LostResponse,
    DuplicatedAcknowledgement,
    /// The external owner decoded the projected payload and refused it. Unlike
    /// the other faults this one is determinate — the effect did not happen —
    /// but it is still not completion, so it classifies as unresolved.
    PayloadRejected,
    UnsupportedProtocolVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    UnknownProviderOutcome,
    /// A transport response was observed, but its causal observation could not
    /// be derived after contact; the performed effect remains unresolved.
    ObservationDerivationDenied,
}

/// First-class classification of an unresolved external effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectClassification {
    fault: ExternalRailTransportFault,
    attempt: ExternalEffectPosture,
    /// Exact trusted-time sample used for the expiry/timeout decision (R8.7).
    decision_time: Option<WorthQueryRuntimeTimeSample>,
}

impl ExternalEffectClassification {
    pub const fn fault(&self) -> ExternalRailTransportFault {
        self.fault
    }

    pub const fn attempt_posture(&self) -> &ExternalEffectPosture {
        &self.attempt
    }

    /// The trusted-time sample this classification decided against.
    pub const fn decision_time(&self) -> Option<&WorthQueryRuntimeTimeSample> {
        self.decision_time.as_ref()
    }

    pub fn is_external_completion(&self) -> bool {
        false
    }
}

/// Classify a transport fault against a dispatch-attempt posture.
///
/// Never upgrades the attempt into `ExternalCompletion`. Timeout decisions
/// record the exact host clock sample in the classification facts.
pub(super) fn classify_transport_fault(
    fault: ExternalRailTransportFault,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ExternalEffectClassification {
    let decision_time = clock
        .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
        .ok();
    ExternalEffectClassification {
        fault,
        attempt: attempt.attempt().clone(),
        decision_time,
    }
}
