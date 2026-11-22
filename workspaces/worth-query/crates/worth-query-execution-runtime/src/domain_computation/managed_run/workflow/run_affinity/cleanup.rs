use std::sync::Arc;

use super::WorthQueryWorkflowRunAffinity;
use crate::domain_computation::managed_run::provider_work::WorthQueryManagedProviderWorkEvidence;
use crate::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupPermit;
use crate::domain_computation::{
    WorthQueryWorkflowExecutionAttemptReleaseReceipt, WorthQueryWorkflowExecutionResourceAttempt,
};

pub struct WorthQueryWorkflowYieldReleasePending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowExecutionResourceAttempt,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

pub struct WorthQueryWorkflowAffinityCleanupReceipt {
    logical: Arc<str>,
    attempt_identity: Arc<str>,
    attempt: WorthQueryWorkflowExecutionAttemptReleaseReceipt,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

impl WorthQueryWorkflowRunAffinity {
    pub fn finish_yield(self) -> WorthQueryWorkflowYieldReleasePending {
        WorthQueryWorkflowYieldReleasePending {
            logical: self.logical,
            attempt: self.attempt,
            provider_work: self.provider_work.into_evidence(),
        }
    }

    pub fn finish_cleanup(
        self,
        _owner: &WorthQueryWorkflowReadmissionCleanupPermit,
    ) -> WorthQueryWorkflowAffinityCleanupReceipt {
        self.finish_yield().release()
    }
}

impl WorthQueryWorkflowYieldReleasePending {
    pub fn release(self) -> WorthQueryWorkflowAffinityCleanupReceipt {
        WorthQueryWorkflowAffinityCleanupReceipt {
            logical: self.logical,
            attempt_identity: Arc::from(self.attempt.attempt_identity().as_str()),
            attempt: self.attempt.release(),
            provider_work: self.provider_work,
        }
    }
}

impl WorthQueryWorkflowAffinityCleanupReceipt {
    pub fn logical_run_identity(&self) -> &str {
        &self.logical
    }

    pub fn yielded_attempt_identity(&self) -> &str {
        &self.attempt_identity
    }

    pub fn attempt(&self) -> &WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        &self.attempt
    }

    pub fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }
}
