use serde::Serialize;

use super::{
    ApplySummary, ChangeInputSummary, FlowCauseSample, FlowSummary, InvalidationSummary,
    PlanningSummary, PrecomputeSummary, RollbackSummary,
};
use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::ExplanationSummary;
use crate::logic::transaction::ObservationBoundarySummary;

/// Borrowed logical flow summary, including its latest retained annotations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename = "FlowSummary")]
pub struct RetainedFlowSummaryView<'a> {
    pub profile: DiagnosticsTier,
    pub change: &'a ChangeInputSummary,
    pub invalidation: &'a InvalidationSummary,
    pub planning: &'a PlanningSummary,
    pub precompute: &'a PrecomputeSummary,
    pub apply: &'a ApplySummary,
    pub cause_samples: &'a [FlowCauseSample],
    pub event_epochs: &'a [EventEpochSummary],
    pub observation: Option<&'a ObservationBoundarySummary>,
    pub rollback: Option<&'a RollbackSummary>,
    pub explanation: Option<&'a ExplanationSummary>,
}

impl RetainedFlowSummaryView<'_> {
    /// Materialize an independently owned export. Cost follows retained detail.
    pub fn to_owned_summary(self) -> FlowSummary {
        FlowSummary {
            profile: self.profile,
            change: self.change.clone(),
            invalidation: self.invalidation.clone(),
            planning: self.planning.clone(),
            precompute: self.precompute.clone(),
            apply: self.apply.clone(),
            cause_samples: self.cause_samples.to_vec(),
            event_epochs: self.event_epochs.to_vec(),
            observation: self.observation.cloned(),
            rollback: self.rollback.cloned(),
            explanation: self.explanation.cloned(),
        }
    }
}
