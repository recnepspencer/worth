//! Current-output selection charges and refuses against the reader's work budget.
use super::*;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub(super) fn require_current_output_budget(
        &mut self,
        required: usize,
        subject: &str,
    ) -> Result<(), WorthQueryCurrentOutputDenial> {
        self.reader
            .work_budget
            .can_afford(required)
            .then_some(())
            .ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })
    }
    pub(super) fn charge_current_output_work(
        &mut self,
        actual: usize,
        subject: &str,
    ) -> Result<(), WorthQueryCurrentOutputDenial> {
        self.reader
            .work_budget
            .consume(actual)
            .then_some(())
            .ok_or_else(|| self.current_output_work_exhausted(subject))
    }

    pub(super) fn current_output_work_exhausted(
        &mut self,
        subject: &str,
    ) -> WorthQueryCurrentOutputDenial {
        self.reader.work_budget.mark_exceeded();
        WorthQueryCurrentOutputDenial::new(
            WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
            subject,
        )
    }
}
