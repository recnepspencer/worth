#[derive(Default)]
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryTemporalOperationTotals
{
    committed: usize,
    already_committed: usize,
    failed: usize,
    indeterminate: usize,
}

impl WorthQueryTemporalOperationTotals {
    pub(super) fn accumulate(
        &mut self,
        counts: super::super::application_operation_reentry::WorthQueryTemporalReentryCounts,
    ) {
        self.committed = self.committed.saturating_add(counts.committed);
        self.already_committed = self
            .already_committed
            .saturating_add(counts.already_committed);
        self.failed = self.failed.saturating_add(counts.failed);
        self.indeterminate = self.indeterminate.saturating_add(counts.indeterminate);
    }

    pub(super) fn apply_to(&self, receipt: &mut super::ErasedClockObservationReceipt) {
        receipt.committed_operation_count = self.committed;
        receipt.already_committed_operation_count = self.already_committed;
        receipt.failed_operation_count = self.failed;
        receipt.indeterminate_operation_count = self.indeterminate;
    }
}
