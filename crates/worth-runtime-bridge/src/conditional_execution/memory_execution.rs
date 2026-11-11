//! Memory-only owner forwards to the same conditional algorithms as authored branches.
use super::*;
use std::sync::Arc;
impl BridgeOwnedSignalRuntime {
    pub fn execute(
        &self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execution_view()
            .execute(execution_request, signal_basis, request, compute_context)
    }

    pub(super) fn execute_with_managed_source_record(
        &self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        managed_source_record: Option<
            crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
        >,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execution_view().execute_with_managed_source_record(
            execution_request,
            signal_basis,
            request,
            managed_source_record,
            compute_context,
        )
    }

    pub fn execute_admitted_conditional(
        &self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        session: &BridgeConditionalEvaluationSession,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execution_view().execute_admitted_conditional(
            execution_request,
            session,
            request,
            compute_context,
        )
    }

    pub fn admit_conditional_evaluation(
        &self,
        request: BridgeConditionalEvaluationAdmissionRequest<'_>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.execution_view()
            .admit_conditional_evaluation(request, execution)
    }

    pub(super) fn admit_conditional_evaluation_with_record(
        &self,
        request: BridgeConditionalEvaluationAdmissionRequest<'_>,
        managed_source_record: Option<
            crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
        >,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.execution_view()
            .admit_conditional_evaluation_with_record(request, managed_source_record, execution)
    }

    pub(super) fn require_live_installed_lowering(
        &self,
        lowering: &std::sync::Arc<super::BridgeInstalledConditionalLowering>,
    ) -> Result<(), super::BridgeConditionalDenial> {
        self.execution_view()
            .require_live_installed_lowering(lowering)
    }

    pub(super) fn admit_exact_conditional_signal_basis(
        &self,
        anchor: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.execution_view()
            .admit_exact_conditional_signal_basis(anchor, basis)
    }

    pub(super) fn admit_conditional_signal_basis(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.execution_view()
            .admit_conditional_signal_basis(lowering, basis)
    }

    pub fn readmit_conditional_evaluation(
        &self,
        request: BridgeConditionalEvaluationReadmissionRequest<'_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.execution_view()
            .readmit_conditional_evaluation(request)
    }

    pub fn reenter_retained_conditional_decision(
        &self,
        request: BridgeConditionalDecisionReentryRequest<'_>,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execution_view()
            .reenter_retained_conditional_decision(request)
    }

    pub fn compare_exact_output(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        baseline_version: u64,
        candidate_version: u64,
    ) -> Result<BridgeExactOutputComparison, BridgeOutputComparisonDenial> {
        self.execution_view().compare_exact_output(
            execution,
            signal_basis,
            request,
            baseline_version,
            candidate_version,
        )
    }
}
