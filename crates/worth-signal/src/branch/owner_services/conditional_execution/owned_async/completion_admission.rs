use super::{ResourceCompletionAdmissionReport, SignalOwnedAsyncSourceBinding};

/// Actual completion admission from one bound conditional service occurrence.
/// Its projected report cannot reconstruct this owner.
pub struct SignalOwnedAsyncCompletionAdmission {
    pub(super) origin: SignalOwnedAsyncSourceBinding,
    pub(super) report: ResourceCompletionAdmissionReport,
}

impl SignalOwnedAsyncCompletionAdmission {
    pub(super) fn new(
        origin: SignalOwnedAsyncSourceBinding,
        report: ResourceCompletionAdmissionReport,
    ) -> Self {
        Self { origin, report }
    }

    pub fn report(&self) -> &ResourceCompletionAdmissionReport {
        &self.report
    }
}

impl std::fmt::Debug for SignalOwnedAsyncCompletionAdmission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalOwnedAsyncCompletionAdmission")
            .field("report", &self.report)
            .finish_non_exhaustive()
    }
}
