//! Terminal validation and exact evidence/receipt completion.

use crate::basis_lifecycle::BasisOperationLane;
use worth_proof::TransitionOutcome;

use super::super::super::{
    WorthQueryBoundExecutionDenial, WorthQueryBoundExecutionDenialKind,
    WorthQueryBoundExecutionReceipt, WorthQueryExecutableDomainOperation,
    WorthQueryOperationOutput,
};
use super::super::receipt_identity::{
    direct_execution_receipt_identity, DirectExecutionIdentityInput,
};
use super::super::{WorthQueryBoundExecutionOutcome, WorthQueryExecutedDomainOperation};
use super::WorthQueryValidatedDirectEvidenceCompletion;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis,
};

impl<D: 'static, O, F: 'static, L: BasisOperationLane, Output: WorthQueryOperationOutput>
    WorthQueryValidatedDirectEvidenceCompletion<D, O, F, L, Output>
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    pub(super) fn finish(mut self) -> WorthQueryBoundExecutionOutcome<D, O, F, L, Output> {
        let output_identity = self.output.operation_output_identity();
        let attachment =
            WorthQueryDirectDomainEvidenceAttachment::new(&self, output_identity.clone());
        let material = self.material.take();
        let domain_evidence =
            match super::super::super::admit_direct_completion_content(attachment, material) {
                Ok(evidence) => evidence,
                Err(denial) => return self.denied(denial),
            };
        let identity = direct_execution_receipt_identity(DirectExecutionIdentityInput {
            binding_identity: self.bound.binding_identity(),
            capability_identity: self.bound.capability_identity(),
            output_identity: &output_identity,
            result_state: self.result_state,
            warnings: &self.warnings,
            graph_receipts: &self.graph_receipts,
            execution_snapshot: &self.snapshot,
            conditional: &self.conditional,
            execution_resources: &self.resource_evidence,
            domain_evidence: domain_evidence.as_ref(),
        });
        let receipt = WorthQueryBoundExecutionReceipt {
            identity,
            binding_identity: self.bound.binding_identity().to_owned(),
            output_identity,
            result_state: self.result_state,
            domain_evidence,
            execution_resources: self.resource_evidence.clone(),
        };
        let phase_proof = mint_operation_phase_proof(
            receipt.identity().to_string(),
            Some(self.phase_proof.payload().identity()),
            operation_phase_basis(&self.phase_proof).clone(),
        );
        let terminal = match self
            .running
            .take()
            .expect("validated direct execution owns its managed run")
            .completed()
        {
            Ok(terminal) => terminal,
            Err(rejection) => {
                let detail = rejection.denial().detail().to_owned();
                return self.managed_run_denied(detail, rejection.into_running());
            }
        };
        let managed_cleanup = match terminal.cleanup() {
            Ok(receipt) => receipt,
            Err(failure) => {
                return TransitionOutcome::Denied(
                    WorthQueryBoundExecutionDenial::new(
                        WorthQueryBoundExecutionDenialKind::GraphProvider,
                        "managed direct cleanup requires owner retry",
                        self.counters,
                    )
                    .with_graph_receipts(self.graph_receipts)
                    .with_managed_cleanup(Err(failure)),
                );
            }
        };
        TransitionOutcome::Success(WorthQueryExecutedDomainOperation {
            bound: self.bound,
            output: self.output,
            receipt,
            warnings: self.warnings,
            counters: self.counters,
            graph_receipts: self.graph_receipts,
            execution_snapshot: self.snapshot,
            phase_proof,
            conditional: self.conditional,
            resources: self.resources,
            managed_cleanup,
        })
    }

    fn denied(
        mut self,
        denial: super::super::super::WorthQueryCompletedDomainEvidenceAdmissionDenial,
    ) -> WorthQueryBoundExecutionOutcome<D, O, F, L, Output> {
        let kind = WorthQueryBoundExecutionDenialKind::DomainEvidence(denial.kind());
        let cleanup = self
            .running
            .take()
            .expect("validated direct execution owns its managed run")
            .abandon()
            .cleanup();
        TransitionOutcome::Denied(
            WorthQueryBoundExecutionDenial::new(kind, denial.subject(), self.counters)
                .with_graph_receipts(self.graph_receipts)
                .with_managed_cleanup(cleanup),
        )
    }

    fn managed_run_denied(
        self,
        detail: String,
        running: worth_query_execution::facade::runtime::WorthQueryRunningDirectRun,
    ) -> WorthQueryBoundExecutionOutcome<D, O, F, L, Output> {
        let cleanup = running.abandon().cleanup();
        TransitionOutcome::Denied(
            WorthQueryBoundExecutionDenial::new(
                WorthQueryBoundExecutionDenialKind::GraphProvider,
                detail,
                self.counters,
            )
            .with_graph_receipts(self.graph_receipts)
            .with_managed_cleanup(cleanup),
        )
    }
}

pub(in crate::domain_installation::operation_execution) struct WorthQueryDirectDomainEvidenceAttachment
{
    operation_identity: String,
    binding_identity: String,
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

impl WorthQueryDirectDomainEvidenceAttachment {
    fn new<D, O, F, L, Output>(
        completion: &WorthQueryValidatedDirectEvidenceCompletion<D, O, F, L, Output>,
        output_occurrence_identity: String,
    ) -> Self
    where
        L: BasisOperationLane,
        O: WorthQueryExecutableDomainOperation<D, F>,
    {
        Self {
            operation_identity: completion
                .bound
                .definition()
                .canonical_identity()
                .to_owned(),
            binding_identity: completion.bound.binding_identity().to_owned(),
            basis_identity: completion.bound.basis_identity().to_owned(),
            execution_snapshot_identity: completion
                .snapshot
                .evidence_identity()
                .as_str()
                .to_owned(),
            contract: completion.bound.direct_domain_evidence_contract(),
            resource_evidence: completion.resource_evidence.clone(),
            graph_receipt_identities: completion
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
