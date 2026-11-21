use std::sync::Arc;

use crate::domain_computation::WorthQueryDirectExecutionResourceAttempt;

pub struct WorthQueryDirectRunTerminalAffinity {
    logical: Arc<str>,
    custody: Option<Arc<super::super::durable_custody::ManagedRunCustody>>,
    attempt: WorthQueryDirectExecutionResourceAttempt,
}

impl WorthQueryDirectRunTerminalAffinity {
    pub(super) const fn new(
        logical: Arc<str>,
        attempt: WorthQueryDirectExecutionResourceAttempt,
        custody: Option<Arc<super::super::durable_custody::ManagedRunCustody>>,
    ) -> Self {
        Self {
            logical,
            attempt,
            custody,
        }
    }

    pub fn custody(&self) -> Option<Arc<super::super::durable_custody::ManagedRunCustody>> {
        self.custody.clone()
    }

    pub fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub fn terminal_descriptions(&self) -> (Arc<str>, Arc<str>) {
        (
            Arc::clone(&self.logical),
            Arc::from(self.attempt.attempt_identity().as_str()),
        )
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryDirectExecutionAttemptReleaseReceipt {
        self.attempt.release()
    }
}
