//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowExecutionResourceAttempt {
    #[doc(hidden)]
    pub fn release_for_test(self) -> WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        Self::release(self)
    }
}
