use std::sync::Arc;

use crate::domain_computation::{
    WorthQueryWorkflowExecutionAttemptReleaseReceipt, WorthQueryWorkflowExecutionResourceAttempt,
};

pub struct WorthQueryWorkflowRunTerminalAffinity
{
    logical: Arc<str>,
    attempt: WorthQueryWorkflowExecutionResourceAttempt,
}

impl WorthQueryWorkflowRunTerminalAffinity {
    pub(super) fn new(
        logical: Arc<str>,
        attempt: WorthQueryWorkflowExecutionResourceAttempt,
    ) -> Self {
        Self { logical, attempt }
    }

    pub fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub fn release(
        self,
    ) -> WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        self.attempt.release()
    }
}
