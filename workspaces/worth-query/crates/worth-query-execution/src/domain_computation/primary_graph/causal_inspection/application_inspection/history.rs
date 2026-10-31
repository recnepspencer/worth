//! Fresh connected diagnostics consume exact-original, disposable Native history.
use super::super::WorthQueryApplicationQueryObservation;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    primary_graph::application_output_demand::OriginalSettledProducerSource,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalCausalObservation, BridgeConditionalHistoryDetail,
};
pub use worth_runtime_bridge::facade::{
    BridgeConditionalHistoryAvailability as WorthQueryConditionalHistoryAvailability,
    BridgeConditionalHistoryDecisionClass as WorthQueryConditionalHistoryDecisionClass,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalHistoryOutcome {
    NotRequested,
    RequestedButNotProvidedBySelectedObservationMode,
    OriginalObservationNotRetained,
    OwnerUnavailable(WorthQueryConditionalHistoryAvailability),
    Provided,
}
/// A copied, admitted history snapshot, bound to its exact readonly Native original.
/// This describes supplier readiness, not an output-equivalence comparison pair.
pub struct WorthQueryMaterializedConditionalHistory {
    original: BridgeConditionalCausalObservation,
    detail: BridgeConditionalHistoryDetail,
}
impl WorthQueryMaterializedConditionalHistory {
    pub fn attempt(&self) -> u64 {
        self.detail.attempt()
    }
    pub fn decision_class(&self) -> WorthQueryConditionalHistoryDecisionClass {
        self.detail.class()
    }
    pub fn output_version_before(&self) -> u64 {
        self.detail.output_version_before()
    }
    pub fn output_version(&self) -> u64 {
        self.detail.output_version()
    }
    pub(super) fn matches(&self, original: &WorthQueryApplicationQueryObservation) -> bool {
        original_native(original).is_some_and(|native| self.original.same_observation(native))
    }
    pub(super) fn native(&self) -> &BridgeConditionalCausalObservation {
        &self.original
    }
}
pub(super) enum HistoryDetail {
    NotRequested,
    Unselected,
    NotRetained,
    Unavailable(WorthQueryConditionalHistoryAvailability),
    Provided(WorthQueryMaterializedConditionalHistory),
}
impl HistoryDetail {
    pub(super) fn outcome(&self) -> WorthQueryConditionalHistoryOutcome {
        use WorthQueryConditionalHistoryOutcome as Outcome;
        match self {
            Self::NotRequested => Outcome::NotRequested,
            Self::Unselected => Outcome::RequestedButNotProvidedBySelectedObservationMode,
            Self::NotRetained => Outcome::OriginalObservationNotRetained,
            Self::Unavailable(cause) => Outcome::OwnerUnavailable(*cause),
            Self::Provided(_) => Outcome::Provided,
        }
    }
    /// The connected caller has already verified both exact joins with these
    /// same live loans before reading the owner's current availability or body.
    pub(super) fn capture(
        original: &WorthQueryApplicationQueryObservation,
        _query: &DiagnosticComparisonAdmission<'_>,
        _source: &DiagnosticComparisonAdmission<'_>,
    ) -> Self {
        let Some(native) = original_native(original) else {
            return Self::NotRetained;
        };
        match native.history_detail() {
            Ok(detail) => Self::Provided(WorthQueryMaterializedConditionalHistory {
                original: native.clone(),
                detail,
            }),
            Err(cause) => Self::Unavailable(cause),
        }
    }
}
pub(super) fn original_native(
    original: &WorthQueryApplicationQueryObservation,
) -> Option<&BridgeConditionalCausalObservation> {
    match original.source.original_settled_producer()?.source() {
        OriginalSettledProducerSource::Committed { conditional, .. } => conditional.as_ref(),
        _ => None,
    }
}
