use crate::basis_lifecycle::BasisOperationLane;
use worth_proof::TransitionOutcome;
pub(in crate::domain_installation::operation_execution) mod evidence_completion;
mod publication_progression;
mod receipt_identity;
use super::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryBoundExecutionDenial,
    WorthQueryBoundExecutionReceipt, WorthQueryBoundGraphExecutionReceipt,
    WorthQueryExecutableDomainOperation, WorthQueryOperationExecutionCounters,
    WorthQueryOperationExecutionWarning, WorthQueryOperationOutput, WorthQueryTerminalOperation,
};
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryExecutedOperationPhase,
    WorthQueryOperationPhaseProof,
};
use crate::domain_installation::WorthQueryBoundDomainOperation;
pub use publication_progression::WorthQueryPublicationDenial;
pub struct WorthQueryExecutedDomainOperation<D, O, F, L: BasisOperationLane, Output> {
    pub(super) bound: WorthQueryBoundDomainOperation<D, O, F, L>,
    pub(super) output: Output,
    receipt: WorthQueryBoundExecutionReceipt,
    warnings: Vec<WorthQueryOperationExecutionWarning>,
    pub(super) counters: WorthQueryOperationExecutionCounters,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    execution_snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    phase_proof: WorthQueryOperationPhaseProof<WorthQueryExecutedOperationPhase>,
    conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    resources: WorthQueryAdmittedExecutionResourcePlan,
    managed_cleanup: worth_query_execution::facade::runtime::WorthQueryDirectRunCleanupReceipt,
}
pub type WorthQueryBoundExecutionOutcome<D, O, F, L, Output> = TransitionOutcome<
    WorthQueryExecutedDomainOperation<D, O, F, L, Output>,
    WorthQueryBoundExecutionDenial,
    crate::domain_installation::WorthQueryDeferredDomainOperation<D, O, F, L>,
    WorthQueryBoundExecutionDenial,
    WorthQueryBoundExecutionDenial,
    WorthQueryBoundExecutionDenial,
>;
impl<D, O, F, L: BasisOperationLane, Output> WorthQueryExecutedDomainOperation<D, O, F, L, Output> {
    pub fn receipt(&self) -> &WorthQueryBoundExecutionReceipt {
        &self.receipt
    }
    pub fn warnings(&self) -> &[WorthQueryOperationExecutionWarning] {
        &self.warnings
    }
    pub fn counters(&self) -> WorthQueryOperationExecutionCounters {
        self.counters
    }
    pub fn graph_receipts(&self) -> &[WorthQueryBoundGraphExecutionReceipt] {
        &self.graph_receipts
    }
    pub fn conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        &self.conditional
    }
    pub fn resources(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        &self.resources
    }
    pub fn provider_session_identity(&self) -> &str {
        self.managed_cleanup
            .inspection()
            .provider_session_identity()
    }
}
impl<D, O, F, L: BasisOperationLane, Output> WorthQueryExecutedDomainOperation<D, O, F, L, Output>
where
    Output: WorthQueryOperationOutput,
{
    /// Closed identity derived from the still-owned executor output.
    pub fn completed_output_occurrence_identity(&self) -> String {
        self.output.operation_output_identity()
    }
}
impl<D, O, F, L: BasisOperationLane, Output> WorthQueryExecutedDomainOperation<D, O, F, L, Output>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Output = Output,
        Publication = WorthQueryTerminalOperation,
        Execution = super::WorthQueryDirectOperation,
    >,
{
    pub fn output(&self) -> &Output {
        &self.output
    }
}
