//! Conditional calls refine the real memory or directory owner at an admitted Signal basis.
use std::sync::{Arc, RwLockReadGuard};
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::{
    BridgeAuthoredConditionalDeclaration, BridgeAuthoredConditionalExecution,
    BridgeConditionalDecisionEvidence, BridgeConditionalDenial, BridgeConditionalExecutionRequest,
    BridgeConditionalSignalBasisBinding, BridgeExactOutputComparison, BridgeOutputComparisonDenial,
    BridgeReadmittedAuthoredAssembly, BridgeSealedRuntimeAssembly,
};
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

pub(in crate::domain_computation) enum ConditionalExecutionOwner<'a> {
    Memory(RwLockReadGuard<'a, BridgeSealedRuntimeAssembly>),
    Authored(&'a Arc<BridgeReadmittedAuthoredAssembly>),
    Stopped,
}
fn unavailable(detail: impl Into<String>) -> BridgeConditionalDenial {
    BridgeConditionalDenial::operation_route_refused(detail)
}
impl ConditionalExecutionOwner<'_> {
    pub(in crate::domain_computation) fn memory(
        &self,
    ) -> Result<&BridgeSealedRuntimeAssembly, BridgeConditionalDenial> {
        match self {
            Self::Memory(owner) => Ok(owner),
            Self::Authored(_) => Err(unavailable(
                "this family requires a later authored-owner recovery phase",
            )),

            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }

    pub(in crate::domain_computation) fn conditional_lifecycle_probe(
        &self,
    ) -> worth_runtime_bridge::facade::BridgeConditionalRuntimeLifecycleProbe {
        match self {
            Self::Memory(owner) => owner.conditional_lifecycle_probe(),
            Self::Authored(owner) => owner.conditional_lifecycle_probe(),
            Self::Stopped => {
                worth_runtime_bridge::facade::BridgeConditionalRuntimeLifecycleProbe::closed()
            }
        }
    }
    pub(in crate::domain_computation) fn installed_conditional_definition_count(&self) -> usize {
        match self {
            Self::Memory(owner) => owner.installed_conditional_definition_count(),
            Self::Authored(owner) => owner.installed_conditional_definition_count(),
            Self::Stopped => 0,
        }
    }
    pub(in crate::domain_computation) fn installed_conditional_target_reference_count(
        &self,
    ) -> usize {
        match self {
            Self::Memory(owner) => owner.installed_conditional_target_reference_count(),
            Self::Authored(owner) => owner.installed_conditional_target_reference_count(),
            Self::Stopped => 0,
        }
    }
    pub(in crate::domain_computation) fn owned_signal_active_node_count(
        &self,
    ) -> Result<usize, BridgeConditionalDenial> {
        match self {
            Self::Memory(owner) => owner.owned_signal_active_node_count(),
            Self::Authored(owner) => owner.owned_signal_active_node_count(),
            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn signal_operation_control_for_test(
        &self,
    ) -> Result<
        worth_signal::facade::branch::SignalOwnerOperationControlForTest,
        BridgeConditionalDenial,
    > {
        match self {
            Self::Memory(owner) => owner
                .signal_operation_control_for_test()
                .map_err(|_| BridgeConditionalDenial::owner_stopped()),
            Self::Authored(owner) => Ok(owner.signal_operation_control_for_test()),
            Self::Stopped => Err(BridgeConditionalDenial::owner_stopped()),
        }
    }
    pub(in crate::domain_computation) fn configure_conditional_history_for_test(
        &self,
        enabled: bool,
        limit: usize,
    ) -> bool {
        match self {
            Self::Memory(owner) => owner.configure_conditional_history_for_test(enabled, limit),
            Self::Authored(owner) => owner.configure_conditional_history_for_test(enabled, limit),
            Self::Stopped => false,
        }
    }
    fn authored_at<'a>(
        owner: &'a BridgeReadmittedAuthoredAssembly,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<BridgeAuthoredConditionalExecution<'a>, BridgeConditionalDenial> {
        let branch = worth_foundational::facade::DurableSignalBranchIdentity::from_recorded(
            owner.signal_owner().binding().owner(),
            basis.branch_id().0,
        );
        owner.conditional_execution(branch).map_err(|cause| {
            unavailable(format!(
                "authored conditional branch is unavailable: {cause:?}"
            ))
        })
    }
    pub(in crate::domain_computation) fn admit_declaration_signal_basis(
        &self,
        declaration: &BridgeAuthoredConditionalDeclaration,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        match self {
            Self::Memory(_) => Err(unavailable(
                "authored declaration cannot select a memory owner",
            )),
            Self::Authored(owner) => {
                Self::authored_at(owner, basis)?.admit_declaration_signal_basis(declaration, basis)
            }

            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn admit_memory_signal_basis(
        &self,
        lowering: &Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        match self {
            Self::Memory(owner) => owner.admit_exact_conditional_signal_basis(lowering, basis),
            Self::Authored(_) => Err(unavailable(
                "memory installation authority cannot select an authored owner",
            )),

            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn deliver_authoritative_change(
        &self,
        execution: ExecutionRequest<'_, '_>,
        basis: &BridgeConditionalSignalBasisBinding,
        ordinal: usize,
        request: worth_runtime_bridge::facade::RelationalCommittedPatchRequest,
    ) -> Result<worth_runtime_bridge::facade::CorrespondenceDeliveryOutcome, BridgeConditionalDenial>
    {
        match self {
            Self::Memory(owner) => {
                owner.deliver_authoritative_change(execution, basis, ordinal, request)
            }
            Self::Authored(owner) => Self::authored_at(owner, basis.signal_basis())?
                .deliver_authoritative_change(execution, basis, ordinal, request),
            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn execute(
        &self,
        execution: ExecutionRequest<'_, '_>,
        basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        match self {
            Self::Memory(owner) => owner.execute(execution, basis, request, context),
            Self::Authored(owner) => Self::authored_at(owner, basis.signal_basis())?
                .execute(execution, basis, request, context),

            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn execute_for_source_record(
        &self,
        execution: ExecutionRequest<'_, '_>,
        basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        match self {
            Self::Memory(owner) => {
                owner.execute_for_source_record(execution, basis, request, record, context)
            }
            Self::Authored(owner) => Self::authored_at(owner, basis.signal_basis())?
                .execute_for_source_record(execution, basis, request, record, context),

            Self::Stopped => Err(unavailable("conditional owner has stopped")),
        }
    }
    pub(in crate::domain_computation) fn compare_exact_output(
        &self,
        execution: ExecutionRequest<'_, '_>,
        basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        before: u64,
        after: u64,
    ) -> Result<BridgeExactOutputComparison, BridgeOutputComparisonDenial> {
        match self {
            Self::Memory(owner) => {
                owner.compare_exact_output(execution, basis, request, before, after)
            }
            Self::Authored(owner) => match Self::authored_at(owner, basis.signal_basis()) {
                Ok(owner) => owner.compare_exact_output(execution, basis, request, before, after),
                Err(cause) => Err(BridgeOutputComparisonDenial::admission(cause)),
            },

            Self::Stopped => Err(BridgeOutputComparisonDenial::admission(unavailable(
                "conditional owner has stopped",
            ))),
        }
    }
}
