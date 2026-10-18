//! Query-owned lookup for one admitted guarded operation.

use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

impl WorthQueryWorkflowAdvanceAdapter {
    #[doc(hidden)]
    pub fn resolve_guarded_operation_custody<Schema, Operation, Input, Scope>(
phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
        completed: Option<&crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody,
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(&runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "workflow-phase"))),
        };
        let execution = &phase_request;

        runtime.resolve_admitted_guarded_workflow_operation_custody(
            execution,
            admission,
            idempotency,
            transition_identity,
            completed,
        )
    }
}
