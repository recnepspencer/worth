//! Transport-fault classification without guessing external completion (R8.24).

use super::causal_event::DispatchAttemptEvent;

pub use worth_query_execution_lineage::facade::application_aftermath::external_effect::classification::{
    ExternalEffectClassification, ExternalRailTransportFault,
};

/// Classify a transport fault against a dispatch-attempt posture.
///
/// Never upgrades the attempt into `ExternalCompletion`. Timeout decisions
/// record the exact host clock sample in the classification facts.
pub(super) fn classify_transport_fault(
    fault: ExternalRailTransportFault,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ExternalEffectClassification {
    ExternalEffectClassification::from_attempt_posture(fault, attempt.attempt().clone(), clock)
}
