use std::sync::Arc;

use crate::domain_computation::WorthQueryDirectExecutionResourceAttempt;

pub(in crate::domain_computation::managed_run) struct WorthQueryDirectRunTerminalAffinity {
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

    pub(in crate::domain_computation::managed_run) fn custody(
        &self,
    ) -> Option<Arc<super::super::durable_custody::ManagedRunCustody>> {
        self.custody.clone()
    }

    pub(in crate::domain_computation::managed_run) fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub(in crate::domain_computation::managed_run) fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub(in crate::domain_computation::managed_run) fn terminal_descriptions(
        &self,
    ) -> (Arc<str>, Arc<str>) {
        (
            Arc::clone(&self.logical),
            Arc::from(self.attempt.attempt_identity().as_str()),
        )
    }

    pub(in crate::domain_computation::managed_run) fn release(
        self,
    ) -> crate::domain_computation::WorthQueryDirectExecutionAttemptReleaseReceipt {
        self.attempt.release()
    }
}
