//! Executable custody of one actual authored branch; no posture lock crosses callbacks.
use super::authored_section::{
    branch_projection::BridgeReadmittedAuthoredAssembly,
    readmission::{backing::Backing, Program},
};
use super::*;
use std::sync::{Arc, RwLock};

pub struct BridgeAuthoredConditionalExecution<'a> {
    signal: &'a worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>,
    program: Arc<Program>,
    lowerings: Arc<RwLock<lowering_registry::BridgeConditionalLoweringRegistry>>,
    retention: Arc<retention::BridgeRetentionLedger>,
    allocations: crate::correspondence::SharedCorrespondenceAllocationRegistry,
    source_owner: Arc<worth_proof::ConditionalSourceObservationOwner>,
    declarations: Arc<RwLock<authored_declaration::DeclarationIndex>>,
    graph_instance_id: u64,
    managed_clock_lanes: Arc<
        std::sync::Mutex<
            std::collections::BTreeMap<
                Arc<str>,
                Arc<std::sync::Mutex<managed_time::BridgeManagedClockLane>>,
            >,
        >,
    >,
    _backing: Arc<Backing>,
}
impl BridgeReadmittedAuthoredAssembly {
    pub fn conditional_execution(
        &self,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
    ) -> Result<BridgeAuthoredConditionalExecution<'_>, BridgeAuthoredBranchDenial> {
        let projection = self.branches.projection(branch)?;
        let serving = projection.serving(branch)?;
        Ok(BridgeAuthoredConditionalExecution {
            signal: &self.signal,
            program: Arc::clone(&self.program),
            lowerings: Arc::clone(&serving.lowerings),
            retention: Arc::clone(&serving.retention),
            allocations: Arc::clone(&serving._allocations),
            source_owner: Arc::clone(&serving.source_owner),
            declarations: Arc::clone(&serving.declarations),
            graph_instance_id: serving.graph_instance_id,
            managed_clock_lanes: Arc::clone(&serving.managed_clock_lanes),
            _backing: Arc::clone(&serving._backing),
        })
    }
}
impl BridgeAuthoredConditionalExecution<'_> {
    pub fn admit_declaration_signal_basis(
        &self,
        declaration: &BridgeAuthoredConditionalDeclaration,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.view()
            .admit_declaration_signal_basis(declaration, basis)
    }

    pub fn view(&self) -> execution_view::BridgeConditionalExecutionView<'_> {
        execution_view::BridgeConditionalExecutionView {
            bridge: &self.program.bridge,
            correspondence_allocations: &self.allocations,
            managed_clock_lanes: &self.managed_clock_lanes,
            declaration_scope: Some((&self.program.installation, &self.declarations)),
            retention: &self.retention,
            conditional_lowerings: &self.lowerings,
            signal_graph_instance_id: self.graph_instance_id,
            conditional_evaluation_budget: self.program.conditional_evaluation_budget,
            source_owner: Some(&self.source_owner),
            durable_owner: Some(self.signal),
        }
    }

    pub fn deliver_authoritative_change(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        self.view().deliver_authoritative_change(
            execution,
            signal_basis,
            dependency_ordinal,
            request,
        )
    }
    pub fn execute(
        &self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.view()
            .execute(execution_request, signal_basis, request, compute_context)
    }

    pub fn execute_for_source_record(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        record: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
        context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.view().execute_with_managed_source_record(
            execution,
            basis,
            request,
            Some(record),
            context,
        )
    }
    pub fn execute_admitted_conditional(
        &self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        session: &BridgeConditionalEvaluationSession,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.view().execute_admitted_conditional(
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
        self.view().admit_conditional_evaluation(request, execution)
    }

    pub fn admit_exact_conditional_signal_basis(
        &self,
        anchor: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.view()
            .admit_exact_conditional_signal_basis(anchor, basis)
    }

    pub fn admit_conditional_signal_basis(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.view().admit_conditional_signal_basis(lowering, basis)
    }

    pub fn readmit_conditional_evaluation(
        &self,
        request: BridgeConditionalEvaluationReadmissionRequest<'_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.view().readmit_conditional_evaluation(request)
    }

    pub fn reenter_retained_conditional_decision(
        &self,
        request: BridgeConditionalDecisionReentryRequest<'_>,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.view().reenter_retained_conditional_decision(request)
    }

    pub fn compare_exact_output(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        baseline_version: u64,
        candidate_version: u64,
    ) -> Result<BridgeExactOutputComparison, BridgeOutputComparisonDenial> {
        self.view().compare_exact_output(
            execution,
            signal_basis,
            request,
            baseline_version,
            candidate_version,
        )
    }
}
