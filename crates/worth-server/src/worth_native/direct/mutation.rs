use worth_query::facade::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryBatchWriteReceiptInspection, WorthQueryWriteReceipt,
    WorthQueryWriteReceiptInspection,
};

use crate::{
    WorthServerDirectContextArtifact, WorthServerQuerySupportPosture, WorthServerResponseEnvelope,
};

#[derive(Debug)]
pub struct WorthServerDirectMutation {
    operation_request: crate::WorthServerOperationRequest,
    plan_proof: crate::WorthServerOperationPlanProof,
    support_posture: WorthServerQuerySupportPosture,
    workspace_name: String,
    handoff_digest: String,
    direct_context: WorthServerDirectContextArtifact,
    mutation_result: WorthServerDirectMutationResult,
    response_envelope: WorthServerResponseEnvelope,
    canonical_digest:
        Result<String, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>,
}

pub(crate) struct WorthServerDirectMutationParts {
    pub(crate) operation_request: crate::WorthServerOperationRequest,
    pub(crate) plan_proof: crate::WorthServerOperationPlanProof,
    pub(crate) support_posture: WorthServerQuerySupportPosture,
    pub(crate) workspace_name: String,
    pub(crate) handoff_digest: String,
    pub(crate) direct_context: WorthServerDirectContextArtifact,
    pub(crate) mutation_result: WorthServerDirectMutationResult,
    pub(crate) response_envelope: WorthServerResponseEnvelope,
}

impl WorthServerDirectMutation {
    pub(crate) fn new(parts: WorthServerDirectMutationParts) -> Self {
        let WorthServerDirectMutationParts {
            operation_request,
            plan_proof,
            support_posture,
            workspace_name,
            handoff_digest,
            direct_context,
            mutation_result,
            response_envelope,
        } = parts;
        let canonical_digest = mutation_result.result_digest().map(|result_digest| {
            format!(
                "worth-server-direct-mutation-v1:{}:{}:{}:{}",
                operation_request.canonical_digest(),
                handoff_digest,
                result_digest,
                mutation_result.inspection_digest()
            )
        });
        Self {
            operation_request,
            plan_proof,
            support_posture,
            workspace_name,
            handoff_digest,
            direct_context,
            mutation_result,
            response_envelope,
            canonical_digest,
        }
    }

    pub fn operation_request(&self) -> &crate::WorthServerOperationRequest {
        &self.operation_request
    }

    pub fn plan_proof(&self) -> &crate::WorthServerOperationPlanProof {
        &self.plan_proof
    }

    pub fn support_posture(&self) -> &WorthServerQuerySupportPosture {
        &self.support_posture
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub fn handoff_digest(&self) -> &str {
        &self.handoff_digest
    }

    pub fn direct_context(&self) -> &WorthServerDirectContextArtifact {
        &self.direct_context
    }

    pub fn mutation_result(&self) -> &WorthServerDirectMutationResult {
        &self.mutation_result
    }

    pub fn response_envelope(&self) -> &WorthServerResponseEnvelope {
        &self.response_envelope
    }

    pub fn canonical_digest(
        &self,
    ) -> Result<&str, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>
    {
        self.canonical_digest.as_deref().map_err(|cause| *cause)
    }
}

#[derive(Debug)]
pub enum WorthServerDirectMutationResult {
    Single {
        receipt: Box<WorthQueryWriteReceipt>,
        inspection: Box<WorthQueryWriteReceiptInspection>,
    },
    Batch {
        receipt: Box<WorthQueryBatchWriteReceipt>,
        inspection: Box<WorthQueryBatchWriteReceiptInspection>,
    },
}

impl WorthServerDirectMutationResult {
    pub fn single_receipt(&self) -> Option<&WorthQueryWriteReceipt> {
        match self {
            Self::Single { receipt, .. } => Some(receipt),
            Self::Batch { .. } => None,
        }
    }

    pub fn single_inspection(&self) -> Option<&WorthQueryWriteReceiptInspection> {
        match self {
            Self::Single { inspection, .. } => Some(inspection),
            Self::Batch { .. } => None,
        }
    }

    pub fn batch_receipt(&self) -> Option<&WorthQueryBatchWriteReceipt> {
        match self {
            Self::Single { .. } => None,
            Self::Batch { receipt, .. } => Some(receipt),
        }
    }

    pub fn batch_inspection(&self) -> Option<&WorthQueryBatchWriteReceiptInspection> {
        match self {
            Self::Single { .. } => None,
            Self::Batch { inspection, .. } => Some(inspection),
        }
    }

    pub fn execution_provenance_digest(&self) -> &str {
        match self {
            Self::Single { receipt, .. } => receipt
                .execution_provenance_chain_digest()
                .expect("direct single mutation should preserve execution provenance"),
            Self::Batch { receipt, .. } => receipt
                .execution_provenance()
                .expect("direct batch mutation should preserve execution provenance")
                .execution_provenance_chain_digest(),
        }
    }

    pub fn result_digest(
        &self,
    ) -> Result<&str, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>
    {
        match self {
            Self::Single { receipt, .. } => Ok(receipt
                .commit_evidence_identity()
                .terminal_projection_for_reporting()),
            Self::Batch { receipt, .. } => receipt.batch_digest(),
        }
    }

    pub fn inspection_digest(&self) -> &str {
        match self {
            Self::Single { inspection, .. } => inspection.inspection_digest(),
            Self::Batch { inspection, .. } => inspection.inspection_digest(),
        }
    }
}
