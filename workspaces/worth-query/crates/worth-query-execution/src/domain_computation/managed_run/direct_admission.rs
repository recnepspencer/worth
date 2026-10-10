use worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary;
use worth_runtime_bridge::facade::{RuntimeBridge, RuntimeBridgeRelationalSource};

use super::admission::{validate_direct_run_head, validate_direct_run_lower};
use super::lower_admission::{
    admit_atomic_lower_execution_basis, admit_managed_lower_execution_basis,
    WorthQueryManagedLowerAdmissionFailureKind, WorthQueryManagedLowerBinding,
};
use super::{
    WorthQueryAdmittedDirectRun, WorthQueryManagedDirectRunAdmissionFailure,
    WorthQueryManagedDirectRunAdmissionFailureKind, WorthQueryManagedTruthReadRequest,
};
use crate::domain_computation::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime,
};

pub struct WorthQueryManagedRunAdmission<'runtime> {
    pub(super) query: &'runtime WorthQueryExecutionRuntime,
    pub(super) bridge: &'runtime RuntimeBridge,
    pub(super) relational: &'runtime RuntimeBridgeRelationalSource,
}

impl WorthQueryExecutionRuntime {
    pub fn managed_run_admission<'runtime>(
        &'runtime self,
        bridge: &'runtime RuntimeBridge,
        relational: &'runtime RuntimeBridgeRelationalSource,
    ) -> WorthQueryManagedRunAdmission<'runtime> {
        WorthQueryManagedRunAdmission {
            query: self,
            bridge,
            relational,
        }
    }
}

impl WorthQueryManagedRunAdmission<'_> {
    pub fn admit_direct(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
        request: WorthQueryManagedTruthReadRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryAdmittedDirectRun, WorthQueryManagedDirectRunAdmissionFailure> {
        self.admit_direct_for_boundary(
            operation,
            resource_attempt,
            request,
            WorthQueryExecutionBoundary::BoundedStep,
            execution,
        )
    }

    /// Admit one synchronous Atomic run without managed step or queue authority.
    pub fn admit_atomic_direct(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
        request: WorthQueryManagedTruthReadRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryAdmittedDirectRun, WorthQueryManagedDirectRunAdmissionFailure> {
        self.admit_direct_for_boundary(
            operation,
            resource_attempt,
            request,
            WorthQueryExecutionBoundary::Atomic,
            execution,
        )
    }

    fn admit_direct_for_boundary(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
        request: WorthQueryManagedTruthReadRequest,
        boundary: WorthQueryExecutionBoundary,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryAdmittedDirectRun, WorthQueryManagedDirectRunAdmissionFailure> {
        let counters = match validate_direct_run_head(self.query, operation, &resource_attempt) {
            Ok(counters) => counters,
            Err(denial) => {
                return Err(WorthQueryManagedDirectRunAdmissionFailure::new(
                    WorthQueryManagedDirectRunAdmissionFailureKind::QueryAuthority,
                    denial.detail(),
                    resource_attempt,
                ));
            }
        };
        if !request.matches_operation_product(operation) {
            return Err(WorthQueryManagedDirectRunAdmissionFailure::new(
                WorthQueryManagedDirectRunAdmissionFailureKind::ProductBasisMismatch,
                "managed direct run selection does not match the operation's exact World product occurrence",
                resource_attempt,
            ));
        }
        let admit_lower = match boundary {
            WorthQueryExecutionBoundary::Atomic => admit_atomic_lower_execution_basis,
            WorthQueryExecutionBoundary::BoundedStep => admit_managed_lower_execution_basis,
        };
        let lower = match admit_lower(
            self.bridge,
            self.relational,
            WorthQueryManagedLowerBinding::new(
                operation.binding_identity(),
                resource_attempt.attempt_identity().as_str(),
                resource_attempt.resources().envelope(),
            ),
            request,
            execution,
        ) {
            Ok(lower) => lower,
            Err(failure) => {
                let kind = match failure.kind {
                    WorthQueryManagedLowerAdmissionFailureKind::BridgeSourceProfile => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::ManagedAuthorityJoin
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RelationalBasis => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::RelationalBasis
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RetentionCapacityExhausted => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::RetentionCapacityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::RetentionIdentityExhausted => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::RetentionIdentityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::SnapshotIdentityExhausted => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::SnapshotIdentityExhausted
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::BridgePlanning => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::BridgePlanning
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::InstalledStepContract
                    }
                    WorthQueryManagedLowerAdmissionFailureKind::BridgeExecutionBasis => {
                        WorthQueryManagedDirectRunAdmissionFailureKind::BridgeExecutionBasis
                    }
                };
                return Err(WorthQueryManagedDirectRunAdmissionFailure::new(
                    kind,
                    failure.detail,
                    resource_attempt,
                ));
            }
        };
        let counters = match validate_direct_run_lower(
            operation,
            &resource_attempt,
            &lower.bridge,
            &lower.relational,
            lower.product_observation.as_ref(),
            counters,
        ) {
            Ok(counters) => counters,
            Err(denial) => {
                return Err(WorthQueryManagedDirectRunAdmissionFailure::new(
                    WorthQueryManagedDirectRunAdmissionFailureKind::ManagedAuthorityJoin,
                    denial.detail(),
                    resource_attempt,
                ));
            }
        };
        Ok(WorthQueryAdmittedDirectRun::new(
            operation,
            resource_attempt,
            lower.bridge,
            lower.relational,
            counters,
        ))
    }
}
