//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationAttemptDenial {
    #[doc(hidden)]
    pub fn native_storage_unavailable_for_test(
        cause: worth_relational::facade::storage::RelationalStorageUnavailable,
        subject: &str,
    ) -> Self {
        Self::native_storage_unavailable(cause, subject)
    }
}

impl WorthQueryApplicationAttemptDenial {
    #[doc(hidden)]
    pub fn query_read_denial_for_test(
        stop: worth_relational::facade::runtime::QueryReadExecutionStop,
        subject: &str,
    ) -> Self {
        Self::query_read_denial(stop, subject)
    }
}

impl WorthQueryApplicationAttemptDenial {
    #[doc(hidden)]
    pub fn relational_read_stopped_for_test(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
        subject: &str,
    ) -> Self {
        Self::relational_read_stopped(cause, subject)
    }
}
