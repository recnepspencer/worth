//! Classification of transport results against one exact dispatch attempt.

use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::causal_event::{
    observe_acknowledgement, observe_completion, DispatchAttemptEvent,
    ExternalAcknowledgementEvent, ExternalCompletionEvent, ExternalEffectPosture,
};
use super::classification::{classify_transport_fault, ExternalRailTransportFault};
use super::dispatch::WorthQueryExternalDispatchPosture;
use super::transport::WorthQueryExternalTransportOutcome;

pub(super) struct ClassifiedDispatchObservation {
    pub posture: WorthQueryExternalDispatchPosture,
    pub observation: Option<ExternalEffectPosture>,
    pub canonical_work: WorthQueryCanonicalWorkEvidence,
}

pub(super) fn classify_dispatch_observation(
    observed: WorthQueryExternalTransportOutcome,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ClassifiedDispatchObservation {
    match observed {
        WorthQueryExternalTransportOutcome::Completed => match observe_completion(attempt) {
            Ok(observation) => completed_observation(observation),
            Err(_) => unresolved_observation(
                ExternalRailTransportFault::ObservationDerivationDenied,
                attempt,
                clock,
            ),
        },
        WorthQueryExternalTransportOutcome::Acknowledged => {
            match observe_acknowledgement(attempt) {
                Ok(observation) => acknowledged_observation(observation),
                Err(_) => unresolved_observation(
                    ExternalRailTransportFault::ObservationDerivationDenied,
                    attempt,
                    clock,
                ),
            }
        }
        other => {
            let fault = match other {
                WorthQueryExternalTransportOutcome::DuplicateAcknowledgement => {
                    ExternalRailTransportFault::DuplicatedAcknowledgement
                }
                WorthQueryExternalTransportOutcome::Rejected => {
                    ExternalRailTransportFault::PayloadRejected
                }
                WorthQueryExternalTransportOutcome::UnsupportedProtocolVersion(unsupported) => {
                    ExternalRailTransportFault::UnsupportedProtocolVersion(unsupported)
                }
                WorthQueryExternalTransportOutcome::TimedOut => ExternalRailTransportFault::Timeout,
                WorthQueryExternalTransportOutcome::Disconnected => {
                    ExternalRailTransportFault::Disconnect
                }
                WorthQueryExternalTransportOutcome::LostResponse => {
                    ExternalRailTransportFault::LostResponse
                }
                WorthQueryExternalTransportOutcome::Completed
                | WorthQueryExternalTransportOutcome::Acknowledged => unreachable!(),
            };
            unresolved_observation(fault, attempt, clock)
        }
    }
}

fn unresolved_observation(
    fault: ExternalRailTransportFault,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ClassifiedDispatchObservation {
    ClassifiedDispatchObservation {
        posture: WorthQueryExternalDispatchPosture::unresolved(classify_transport_fault(
            fault, attempt, clock,
        )),
        observation: None,
        canonical_work: WorthQueryCanonicalWorkEvidence::zero(),
    }
}

fn completed_observation(
    observed: (ExternalCompletionEvent, WorthQueryCanonicalWorkEvidence),
) -> ClassifiedDispatchObservation {
    let (observation, canonical_work) = observed;
    let projection = observation.posture().clone();
    ClassifiedDispatchObservation {
        posture: WorthQueryExternalDispatchPosture::completed(observation),
        observation: Some(projection),
        canonical_work,
    }
}

fn acknowledged_observation(
    observed: (
        ExternalAcknowledgementEvent,
        WorthQueryCanonicalWorkEvidence,
    ),
) -> ClassifiedDispatchObservation {
    let (observation, canonical_work) = observed;
    let projection = observation.posture().clone();
    ClassifiedDispatchObservation {
        posture: WorthQueryExternalDispatchPosture::acknowledged(observation),
        observation: Some(projection),
        canonical_work,
    }
}
