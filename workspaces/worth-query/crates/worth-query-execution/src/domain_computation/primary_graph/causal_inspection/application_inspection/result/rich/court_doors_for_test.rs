//! Sealed test doors over this module's private members.
use super::*;

impl CompletedInspection {
    #[doc(hidden)]
    pub fn publish_rich_for_test<'o>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: &'o WorthQueryApplicationQueryObservation,
        report: FundedRichInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedRichInspection<'o>, InspectionAdmissionDenial> {
        Self::publish_rich(self, execution, original, report, memory)
    }
}
