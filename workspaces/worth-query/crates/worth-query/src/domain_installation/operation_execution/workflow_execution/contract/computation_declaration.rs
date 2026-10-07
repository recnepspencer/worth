use super::computation::WorthQueryWorkflowStageTask;
use worth_execution::ChargedBytes;

#[derive(Debug, Default)]
pub(super) struct WorkflowStageComputationDeclaration {
    work: u64,
    scratch_bytes: u64,
    payload_bytes: Option<u64>,
}

impl WorthQueryWorkflowStageTask {
    /// Declare total checkpoint work, peak scratch bytes, and retained result
    /// payload bytes (including a failure's class and detail). Identity storage is carried
    /// and counted separately. Default identity compute declares zero work and
    /// scratch, and retains at most the task's existing payload allocation.
    pub fn with_computation_limits(
        mut self,
        work: u64,
        scratch_bytes: u64,
        payload_bytes: u64,
    ) -> Self {
        self.declaration = WorkflowStageComputationDeclaration {
            work,
            scratch_bytes,
            payload_bytes: Some(payload_bytes),
        };
        self
    }

    pub(in crate::domain_installation::operation_execution) fn declared_work(&self) -> u64 {
        self.declaration.work
    }
    pub(in crate::domain_installation::operation_execution) fn scratch_bytes(&self) -> u64 {
        self.declaration.scratch_bytes
    }
    pub(in crate::domain_installation::operation_execution) fn result_capacity(
        &self,
    ) -> Option<u64> {
        self.identity.additional_charged_bytes().checked_add(
            self.declaration
                .payload_bytes
                .unwrap_or_else(|| self.payload.additional_charged_bytes()),
        )
    }
}
