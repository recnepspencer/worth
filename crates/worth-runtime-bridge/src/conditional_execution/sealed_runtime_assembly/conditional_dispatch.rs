use super::super::BridgeConditionalDenial;
use super::BridgeSealedRuntimeAssembly;

impl BridgeSealedRuntimeAssembly {
    pub fn execute_admitted_conditional(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        session: &super::super::BridgeConditionalEvaluationSession,
        request: super::super::BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime
            .execute_admitted_conditional(execution, session, request, compute_context)
    }

    pub fn execute_managed_due_wake(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        request: super::super::BridgeManagedConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime
            .execute_managed_due_wake(execution, request, compute_context)
    }

    pub fn deliver_authoritative_change(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,

        signal_basis: &super::super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        self.runtime.deliver_authoritative_change(
            execution,
            signal_basis,
            dependency_ordinal,
            request,
        )
    }

    pub fn execute(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::super::BridgeConditionalSignalBasisBinding,
        request: super::super::BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime
            .execute(execution, signal_basis, request, compute_context)
    }

    pub fn execute_for_source_record(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::super::BridgeConditionalSignalBasisBinding,
        request: super::super::BridgeConditionalExecutionRequest<'_>,
        source_record: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<super::super::BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.runtime.execute_with_managed_source_record(
            execution,
            signal_basis,
            request,
            Some(source_record),
            compute_context,
        )
    }
}
