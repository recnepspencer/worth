//! Only fresh final checks can hand the actual session to its release terminal.
use super::super::result::CompletedInspection;
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
    WorthQueryApplicationBasisReleaseReceipt,
};
use worth_execution::MapKernelFailure;
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct InspectionTerminal<
    'storage,
> {
    session: WorthQueryManagedGraphWorkSession,
    proof: WorthQuerySessionInspectionReadProof,
    basis: WorthQueryApplicationBasisIdentity,
    request: WorthQueryRequestScope,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    governance: WorthQueryApplicationQueryGovernance,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
}

impl<'storage, Schema: ApplicationSchema, Scope> InspectionRead<'_, 'storage, Schema, Scope> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn terminal(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<InspectionTerminal<'storage>, InspectionAdmissionDenial> {
        self.check(execution)?;
        Ok(InspectionTerminal {
            session: self.session,
            proof: self
                .proof
                .expect("successful Inspection currentness retains its own proof"),
            basis: self.basis,
            request: self.request.clone(),
            authorization: self.authorization,
            governance: self.governance,
            memory: self.memory,
        })
    }
}

impl InspectionTerminal<'_> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn complete(
        self,
        execution: &ExecutionRequest<'_, '_>,
        release: WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<CompletedInspection, InspectionAdmissionDenial> {
        require_request(execution, &self.request)?;
        let completed = run_retained_stage(execution, self.memory, |context, memory| {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let bytes = self
                .basis
                .retained_clone_heap_bytes()
                .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory {
                memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
            }
            Ok(self
                .session
                .complete_inspection_read(execution, self.proof, release))
        })
        .map_err(InspectionAdmissionDenial::Storage)?
        .map_err(InspectionAdmissionDenial::Terminal)?;
        // No session/Product/basis/proof is returned to the descriptive artifact.
        drop(self.authorization);
        drop(self.governance);
        drop(self.basis);
        require_request(execution, &self.request)?;
        Ok(CompletedInspection::from_terminal(completed, self.request))
    }
}
