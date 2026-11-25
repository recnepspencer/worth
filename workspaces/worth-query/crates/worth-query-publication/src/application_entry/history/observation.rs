//! Forward only the actual document receiver's passive storage observation.
use super::*;

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
{
    #[doc(hidden)]
    pub fn observe_history_enrollment_for_test(
        &mut self,
    ) -> worth_query_execution::publication_boundary::document_history::WorthQueryHistoryEnrollmentObservationForTest{
        self.owner.observe_history_enrollment_for_test()
    }

    /// Rewrites the fact `step` posted outside any navigation and reads it
    /// back; `Err` is the canonical read's refusal (ruling (u)).
    #[doc(hidden)]
    pub fn rewrite_posted_fact_for_test(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        step: &worth_query_execution::publication_boundary::document_history::WorthQueryHistoryNodeId,
    ) -> Result<(), String> {
        self.owner.rewrite_posted_fact_for_test(phase, step).map_err(|denial| format!("{denial:?}"))
    }

    /// The reversal facts live on the document's branch (ruling (u)).
    #[doc(hidden)]
    pub fn reversal_facts_for_test(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Vec<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryReversalFactForTest>
    {
        self.owner.reversal_facts_for_test(phase)
    }
}
