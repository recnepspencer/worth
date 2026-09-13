//! Exact direct-execution completion owner.

use crate::basis_lifecycle::BasisOperationLane;

use super::super::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryBoundGraphExecutionReceipt,
    WorthQueryExecutableDomainOperation, WorthQueryOperationExecutionCounters,
    WorthQueryOperationExecutionWarning,
};

mod completion;
mod execution;

pub(in crate::domain_installation::operation_execution) use completion::WorthQueryDirectDomainEvidenceAttachment;

struct WorthQueryValidatedDirectEvidenceCompletion<D, O, F, L, Output>
where
    L: BasisOperationLane,
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    bound: crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    phase_proof: crate::domain_installation::operation_authority_chain::WorthQueryOperationPhaseProof<
        crate::domain_installation::operation_authority_chain::WorthQueryResourceAdmittedOperationPhase,
    >,
    running: Option<worth_query_execution::facade::runtime::WorthQueryRunningDirectRun>,
    resources: WorthQueryAdmittedExecutionResourcePlan,
    output: Output,
    result_state: crate::domain_installation::WorthQueryOperationResultState,
    warnings: Vec<WorthQueryOperationExecutionWarning>,
    material: Option<super::super::WorthQueryDomainEvidenceMaterial>,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    resource_evidence: super::super::WorthQueryExecutionResourceAttemptEvidence,
    counters: WorthQueryOperationExecutionCounters,
}
