//! Capture-only memory narrowing preserves original request lineage and controls.
use super::WorthQueryCheckpointTransitionResources;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease, LeaseDenial, LeaseRequest,
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
                let parent = parent.ok_or(WorkCeilingDenial::Admission(
                    LeaseDenial::NoActiveExecutionScope,
                ))?;
                narrow(parent)
            })??,
        };
        Ok(Some(lease))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_backing_without_a_lease_refuses_capture_narrowing() {
        let serial = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let resources = WorthQueryCheckpointTransitionResources::bounded(1, 1, 1)
            .unwrap()
            .with_capture_memory_limit(0);
        assert!(matches!(
            resources.capture_lease(
                ExecutionRequest::serial(&serial),
                ExecutionAllocationPolicy::SystemAllocation,
            ),
            Err(WorkCeilingDenial::Admission(
                LeaseDenial::NoActiveExecutionScope
            ))
        ));
    }
}
