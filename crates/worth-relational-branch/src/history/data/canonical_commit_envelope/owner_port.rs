//! Owner access to CanonicalCommitEnvelope across the Relational crate split.

use super::live_managed_source_history::SelectedManagedSourceHistory;
use super::recorded_inputs::RecordedCanonicalInputs;
use super::CanonicalCommitEnvelope;
use crate::validation::candidate_source::authored_history::PreparedSourceImpactHistory;
use std::sync::Arc;

#[doc(hidden)]
pub trait CanonicalCommitEnvelopeOwnerPort: Sized {
    fn semantic_result_arc(
        &self,
    ) -> std::sync::Arc<
        crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    >;
    fn recorded_original_inputs(&self) -> Option<&RecordedCanonicalInputs>;
    fn seal_recorded_original_inputs(&self);
    fn retain_recorded_original_inputs(
        &self,
        inputs: RecordedCanonicalInputs,
    ) -> Result<(), &'static str>;
    fn select_managed_source_history(
        self,
        packet: Option<Arc<PreparedSourceImpactHistory>>,
    ) -> Result<SelectedManagedSourceHistory, Arc<Self>>;
}

impl CanonicalCommitEnvelopeOwnerPort for CanonicalCommitEnvelope {
    fn semantic_result_arc(
        &self,
    ) -> std::sync::Arc<
        crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    > {
        self.semantic_result_arc()
    }

    fn recorded_original_inputs(&self) -> Option<&RecordedCanonicalInputs> {
        self.recorded_original_inputs()
    }

    fn seal_recorded_original_inputs(&self) {
        self.seal_recorded_original_inputs()
    }

    fn retain_recorded_original_inputs(
        &self,
        inputs: RecordedCanonicalInputs,
    ) -> Result<(), &'static str> {
        self.retain_recorded_original_inputs(inputs)
    }

    fn select_managed_source_history(
        self,
        packet: Option<Arc<PreparedSourceImpactHistory>>,
    ) -> Result<SelectedManagedSourceHistory, Arc<Self>> {
        self.select_managed_source_history(packet)
    }
}
