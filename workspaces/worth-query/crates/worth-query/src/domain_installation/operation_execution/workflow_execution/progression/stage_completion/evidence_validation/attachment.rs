//! Closed projection minted only from an exactly validated workflow stage.

use crate::basis_lifecycle::BasisOperationLane;

use super::super::{WorthQueryWorkflowRun, WorthQueryWorkflowStageCompletion};

pub(in crate::domain_installation::operation_execution) struct WorthQueryWorkflowDomainEvidenceAttachment
{
    operation_identity: String,
    binding_identity: String,
    run_identity: String,
    stage_identity: String,
    basis_identity: String,
    execution_snapshot_identity: String,
    contract: Option<
        std::sync::Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
    >,
    resource_evidence: super::super::super::WorthQueryExecutionResourceAttemptEvidence,
    graph_receipt_identities: Vec<String>,
    output_occurrence_identity: String,
}

impl WorthQueryWorkflowDomainEvidenceAttachment {
    pub(super) fn from_completion<D, O, F, L>(
        run: &WorthQueryWorkflowRun<D, O, F, L>,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &WorthQueryWorkflowStageCompletion,
        output_occurrence_identity: String,
    ) -> Self
    where
        L: BasisOperationLane,
    {
        Self {
            operation_identity: run.bound.definition().canonical_identity().to_owned(),
            binding_identity: run.bound.binding_identity().to_owned(),
            run_identity: run.identity.clone(),
            stage_identity: stage.identity().to_owned(),
            basis_identity: run.bound.basis_identity().to_owned(),
            execution_snapshot_identity: input
                .execution_snapshot
                .evidence_identity()
                .as_str()
                .to_owned(),
            contract: run.bound.workflow_stage_domain_evidence_contract(stage),
            resource_evidence: input.resource_evidence.clone(),
            graph_receipt_identities: input
                .graph_receipts
                .iter()
                .map(|receipt| receipt.evidence_identity().to_owned())
                .collect(),
            output_occurrence_identity,
        }
    }

    pub(in crate::domain_installation::operation_execution) fn contract(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>
    {
        self.contract.as_deref()
    }

    pub(in crate::domain_installation::operation_execution) fn operation_identity(&self) -> &str {
        &self.operation_identity
    }

    pub(in crate::domain_installation::operation_execution) fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub(in crate::domain_installation::operation_execution) fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub(in crate::domain_installation::operation_execution) fn stage_identity(&self) -> &str {
        &self.stage_identity
    }

    pub(in crate::domain_installation::operation_execution) fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub(in crate::domain_installation::operation_execution) fn execution_snapshot_identity(
        &self,
    ) -> &str {
        &self.execution_snapshot_identity
    }

    pub(in crate::domain_installation::operation_execution) fn output_occurrence_identity(
        &self,
    ) -> &str {
        &self.output_occurrence_identity
    }

    pub(in crate::domain_installation::operation_execution) fn provider_session_identity(
        &self,
    ) -> &str {
        self.resource_evidence.provider_session_identity()
    }

    pub(in crate::domain_installation::operation_execution) fn provider_session_attempt_identity(
        &self,
    ) -> &str {
        self.resource_evidence.provider_session_attempt_identity()
    }

    pub(in crate::domain_installation::operation_execution) fn graph_receipt_identities(
        &self,
    ) -> impl Iterator<Item = String> + '_ {
        self.graph_receipt_identities.iter().cloned()
    }
}
