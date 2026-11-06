//! Sealed test doors over this module's private members.
use super::*;

impl CompletedInspection {
    #[doc(hidden)]
    pub fn assert_completion_facts_for_test(
        &mut self,
        other: &mut Self,
        execution: &ExecutionRequest<'_, '_>,
        report: &FundedInspectionReport,
    ) {
        Self::assert_completion_facts(self, other, execution, report)
    }
}

impl CompletedInspection {
    #[doc(hidden)]
    pub fn publish_for_test<'original>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: &'original WorthQueryApplicationQueryObservation,
        report: FundedInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedObservedInspection<'original>, InspectionAdmissionDenial> {
        Self::publish(self, execution, original, report, memory)
    }
}
