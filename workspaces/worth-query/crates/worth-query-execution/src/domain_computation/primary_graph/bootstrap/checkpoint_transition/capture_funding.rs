//! Capture-only memory narrowing preserves original request lineage and controls.
use super::WorthQueryCheckpointTransitionResources;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
    WorkCeilingDenial,
};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

impl WorthQueryCheckpointTransitionResources {
    pub(in crate::domain_computation::primary_graph) fn capture_memory_limit_is_set(self) -> bool {
        self.maximum_capture_bytes.is_some()
    }

    pub(in crate::domain_computation::primary_graph) fn capture_lease<'scope, 'authority>(
        self,
        request: ExecutionRequest<'scope, 'authority>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Result<Option<ExecutionResourceLease<'authority>>, WorkCeilingDenial> {
        let Some(bytes) = self.maximum_capture_bytes else {
            return Ok(None);
        };
        let narrow = |parent: &ExecutionResourceLease<'authority>| {
            parent
                .child(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        parent.policy().posture(),
                        parent.policy().determinism(),
                        ExecutionBudget::new(
                            parent.policy().max_workers(),
                            Some(bytes),
                            parent.policy().work_ceiling(),
                        ),
                    ),
                    cancellation: worth_execution::CancellationToken::new(),
                    deadline: None,
                })
                .map_err(WorkCeilingDenial::Admission)
        };
        let lease = match policy {
            ExecutionAllocationPolicy::Execution(parent) => narrow(parent)?,
            ExecutionAllocationPolicy::SystemAllocation => request.in_scope(|parent| {
                narrow(parent.expect("capture narrowing requires admitted leased backing"))
            })??,
        };
        Ok(Some(lease))
    }
}
