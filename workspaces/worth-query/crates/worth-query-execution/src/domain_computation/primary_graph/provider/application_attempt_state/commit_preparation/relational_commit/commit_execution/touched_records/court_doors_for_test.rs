//! Sealed test doors over this module's private members.
use super::*;

impl PreparedTouchedRecords {
    #[doc(hidden)]
    pub fn records_for_test(&self) -> &Arc<ExecutionArray<WorthQueryTouchedRecordIdentity>> {
        &self.records
    }
}

impl PreparedTouchedRecords {
    #[doc(hidden)]
    pub fn prepare_for_test(
        candidate: &PreparedRelationalCommitCandidate,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryProviderSessionCommitStop> {
        Self::prepare(candidate, allocation_policy)
    }
}

impl PreparedTouchedRecords {
    #[doc(hidden)]
    pub fn verify_performed_for_test(
        self,
        exact: &[RecordRef],
    ) -> Arc<ExecutionArray<WorthQueryTouchedRecordIdentity>> {
        Self::verify_performed(self, exact)
    }
}
