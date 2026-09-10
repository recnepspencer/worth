use super::admission::{validate_workflow_run_head, validate_workflow_run_lower};
use super::lower_admission::{
    admit_managed_lower_execution_basis, WorthQueryManagedLowerAdmissionFailureKind,
    WorthQueryManagedLowerBinding,
};
use super::{
    WorthQueryAdmittedWorkflowRun, WorthQueryManagedRunAdmission,
    WorthQueryManagedTruthReadRequest, WorthQueryManagedWorkflowRunAdmissionFailure,
    WorthQueryManagedWorkflowRunAdmissionFailureKind,
};
use crate::domain_computation::{
    WorthQueryExecutionBoundOperationAuthority, WorthQueryWorkflowExecutionResourceAttempt,
};

impl WorthQueryManagedRunAdmission<'_> {
    pub fn admit_workflow(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryWorkflowExecutionResourceAttempt,
        request: WorthQueryManagedTruthReadRequest,
    ) -> Result<WorthQueryAdmittedWorkflowRun, WorthQueryManagedWorkflowRunAdmissionFailure> {
        let counters = match validate_workflow_run_head(self.query, operation, &resource_attempt) {
            Ok(counters) => counters,
            Err(denial) => {
                return Err(WorthQueryManagedWorkflowRunAdmissionFailure::new(
                    WorthQueryManagedWorkflowRunAdmissionFailureKind::QueryAuthority,
                    denial.detail(),
                    resource_attempt,
                ));
            }
        };
        if !request.matches_operation_product(operation) {
            return Err(WorthQueryManagedWorkflowRunAdmissionFailure::new(
                WorthQueryManagedWorkflowRunAdmissionFailureKind::ProductBasisMismatch,
                "managed workflow run selection does not match the operation's exact World product occurrence",
                resource_attempt,
            ));
        }
        let lower = match admit_managed_lower_execution_basis(
            self.bridge,
            self.relational,
            WorthQueryManagedLowerBinding::new(
                operation.binding_identity(),
                resource_attempt.attempt_identity().as_str(),
                resource_attempt.operation_resources().envelope(),
            ),
            request,
        ) {
            Ok(lower) => lower,
            Err(failure) => {
                let kind = match failure.kind {
                    WorthQueryManagedLowerAdmissionFailureKind::BridgeSourceProfile => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::ManagedAuthorityJoin
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RelationalBasis => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::RelationalBasis
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RetentionCapacityExhausted => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::RetentionCapacityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RetentionIdentityExhausted => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::RetentionIdentityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::SnapshotIdentityExhausted => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::SnapshotIdentityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::BridgePlanning => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::BridgePlanning
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::InstalledStepContract
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::BridgeExecutionBasis => {
                        WorthQueryManagedWorkflowRunAdmissionFailureKind::BridgeExecutionBasis
                    }
                };
                return Err(WorthQueryManagedWorkflowRunAdmissionFailure::new(
                    kind,
                    failure.detail,
                    resource_attempt,
                ));
            }
        };
        let counters = match validate_workflow_run_lower(
            operation,
            &resource_attempt,
            &lower.bridge,
            &lower.relational,
            lower.product_observation.as_ref(),
            counters,
        ) {
            Ok(counters) => counters,
            Err(denial) => {
                return Err(WorthQueryManagedWorkflowRunAdmissionFailure::new(
                    WorthQueryManagedWorkflowRunAdmissionFailureKind::ManagedAuthorityJoin,
                    denial.detail(),
                    resource_attempt,
                ));
            }
        };
        Ok(WorthQueryAdmittedWorkflowRun::new(
            operation,
            resource_attempt,
            lower.bridge,
            lower.relational,
            counters,
        ))
    }
}
