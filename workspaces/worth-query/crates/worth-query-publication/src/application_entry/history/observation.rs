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
}
