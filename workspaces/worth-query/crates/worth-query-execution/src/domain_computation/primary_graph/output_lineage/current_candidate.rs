//! Complete candidate custody from the actual selected native publication.
use super::*;
impl WorthQueryCurrentOutputCandidate {
    pub(super) fn from_recorded(recorded: &RecordedOutput, role: &str) -> Option<Self> {
        Some(Self {
            consumed_outputs: Arc::clone(&recorded.consumed_outputs),
            verification_requirement: recorded.verification_requirement(),
            settlement_identity: Arc::clone(&recorded.settlement_identity),
            correspondence: Arc::clone(&recorded.correspondence),
            output_role: role.to_owned(),
            observed_source_facts: recorded.observed_source_facts()?,
            native_output_witness: recorded
                .performed_origin
                .as_ref()
                .and_then(|origin| origin.get())
                .unwrap_or(recorded)
                .native_output_witness_cell()
                .map(Arc::clone),
        })
    }
}
