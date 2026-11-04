//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationAttemptDenial {
    #[doc(hidden)]
    pub fn allocation_denied_for_test(
        subject: impl Into<String>,
        denial: worth_execution::ExecutionAllocationDenial,
    ) -> Self {
        Self::allocation_denied(subject, denial)
    }
}

impl WorthQueryApplicationAttemptDenial {
    #[doc(hidden)]
    pub fn relational_execution_cause_for_test(
        &self,
    ) -> Option<worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        Self::relational_execution_cause(self)
    }
}
