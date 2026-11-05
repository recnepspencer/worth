//! Moved whole from `domain_computation::provider_session::direct_attempt`: the items below and every impl they own.
use worth_query_admission::integration::WorthQueryExecutionCapacityReleaseReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDirectExecutionAttemptReleaseReceipt {
    pub(in crate::domain_computation) provider_session_identity: String,
    pub(in crate::domain_computation) capacity: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryDirectExecutionAttemptReleaseReceipt {
    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn capacity(&self) -> &WorthQueryExecutionCapacityReleaseReceipt {
        &self.capacity
    }
}
