//! Query-owned lookup for one admitted guarded operation.

use super::*;

impl WorthQueryWorkflowAdvanceAdapter {
    #[doc(hidden)]
    pub fn resolve_guarded_operation_custody<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody,
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.resolve_admitted_guarded_workflow_operation_custody(
            admission,
            idempotency,
            transition_identity,
        )
    }
}
