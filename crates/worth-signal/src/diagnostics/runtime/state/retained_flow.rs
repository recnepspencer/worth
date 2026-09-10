mod retained_charge;

use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::flow::{
    ApplySummary, ChangeInputSummary, FlowCauseSample, FlowSummary, InvalidationSummary,
    PlanningSummary, PrecomputeSummary, RetainedFlowSummaryView, RollbackSummary,
};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::ExplanationSummary;
use crate::logic::transaction::ObservationBoundarySummary;

/// Retention roots for one completed flow and its independently replaced annotations.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RetainedFlow {
    payload: Arc<FlowPayload>,
    event_epochs: Arc<Vec<EventEpochSummary>>,
    observation: Option<Arc<ObservationBoundarySummary>>,
}

#[derive(Debug, PartialEq)]
struct FlowPayload {
    profile: DiagnosticsTier,
    change: ChangeInputSummary,
    invalidation: InvalidationSummary,
    planning: PlanningSummary,
    precompute: PrecomputeSummary,
    apply: ApplySummary,
    cause_samples: Vec<FlowCauseSample>,
    rollback: Option<RollbackSummary>,
    explanation: Option<ExplanationSummary>,
}

impl From<FlowSummary> for RetainedFlow {
    fn from(summary: FlowSummary) -> Self {
        let FlowSummary {
            profile,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            event_epochs,
            observation,
            rollback,
            explanation,
        } = summary;
        Self {
            payload: Arc::new(FlowPayload {
                profile,
                change,
                invalidation,
                planning,
                precompute,
                apply,
                cause_samples,
                rollback,
                explanation,
            }),
            event_epochs: Arc::new(event_epochs),
            observation: observation.map(Arc::new),
        }
    }
}

impl RetainedFlow {
    pub(super) fn view(&self) -> RetainedFlowSummaryView<'_> {
        let FlowPayload {
            profile,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            rollback,
            explanation,
        } = self.payload.as_ref();
        RetainedFlowSummaryView {
            profile: *profile,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            event_epochs: &self.event_epochs,
            observation: self.observation.as_deref(),
            rollback: rollback.as_ref(),
            explanation: explanation.as_ref(),
        }
    }

    pub(super) fn record_observation(&mut self, observation: Arc<ObservationBoundarySummary>) {
        self.observation = Some(observation);
    }

    pub(super) fn attach_event_epochs(&mut self, event_epochs: Vec<EventEpochSummary>) {
        self.event_epochs = Arc::new(event_epochs);
    }
}

impl Serialize for RetainedFlow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.view().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RetainedFlow {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        FlowSummary::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
#[path = "retained_flow_tests.rs"]
mod tests;
