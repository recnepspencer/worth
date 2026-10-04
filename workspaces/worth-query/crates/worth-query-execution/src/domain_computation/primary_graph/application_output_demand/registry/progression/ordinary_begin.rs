//! The ordinary entry's registry transitions: begin a row's next stage, or
//! rejoin an output at the checkpoint the same call just published or moved.

use super::*;

const CLAIM_OVERFLOW: &str = "output advancement claim identity exhausted";

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn begin(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> WorthQueryOutputDemandAdvanceAdmission {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("live demand interest retains its owner record");
        begin_record(record, CLAIM_OVERFLOW)
    }

    /// Claim the next checkpoint stage of a published output, or read its
    /// Ready. A row that is no longer a published output belongs to whoever
    /// reopened it, so this caller waits.
    pub(in crate::domain_computation::primary_graph) fn begin_published(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> WorthQueryOutputDemandAdvanceAdmission {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.records.get_mut(&interest.key) {
            Some(record) if matches!(record.state, DemandState::Output(_)) => {
                begin_record(record, CLAIM_OVERFLOW)
            }
            _ => WorthQueryOutputDemandAdvanceAdmission::Pending,
        }
    }
}
