use std::sync::Arc;

use super::WorthQueryWorkflowRunAffinity;
use crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence;
use crate::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupPermit;
use crate::domain_computation::{
    WorthQueryWorkflowExecutionAttemptReleaseReceipt, WorthQueryWorkflowExecutionResourceAttempt,
};

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowYieldReleasePending {
    logical: Arc<str>,
    attempt: WorthQueryWorkflowExecutionResourceAttempt,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowAffinityCleanupReceipt {
    logical: Arc<str>,
    attempt_identity: Arc<str>,
    attempt: WorthQueryWorkflowExecutionAttemptReleaseReceipt,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

impl WorthQueryWorkflowRunAffinity {
    pub(in crate::domain_computation::managed_run) fn finish_yield(
        self,
    ) -> WorthQueryWorkflowYieldReleasePending {
        WorthQueryWorkflowYieldReleasePending {
            logical: self.logical,
            attempt: self.attempt,
            provider_work: self.provider_work.into_evidence(),
        }
    }

    pub(in crate::domain_computation::managed_run) fn finish_cleanup(
        self,
        _owner: &WorthQueryWorkflowReadmissionCleanupPermit,
    ) -> WorthQueryWorkflowAffinityCleanupReceipt {
        self.finish_yield().release()
    }
}

impl WorthQueryWorkflowYieldReleasePending {
    pub(in crate::domain_computation::managed_run) fn release(
        self,
    ) -> WorthQueryWorkflowAffinityCleanupReceipt {
        WorthQueryWorkflowAffinityCleanupReceipt {
            logical: self.logical,
            attempt_identity: Arc::from(self.attempt.attempt_identity().as_str()),
            attempt: self.attempt.release(),
            provider_work: self.provider_work,
        }
    }
}

impl WorthQueryWorkflowAffinityCleanupReceipt {
    pub(in crate::domain_computation::managed_run) fn logical_run_identity(&self) -> &str {
        &self.logical
    }

    pub(in crate::domain_computation::managed_run) fn yielded_attempt_identity(&self) -> &str {
        &self.attempt_identity
    }

    pub(in crate::domain_computation::managed_run) fn attempt(
        &self,
    ) -> &WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        &self.attempt
    }

    pub(in crate::domain_computation::managed_run) fn provider_work(
        &self,
    ) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }
}
