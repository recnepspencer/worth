use worth_query::facade::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryBatchWriteReceiptInspection, WorthQueryWriteReceipt,
    WorthQueryWriteReceiptInspection,
};

use super::{
    envelope::WorthServerCompatibilityMutationEnvelope,
    precondition::WorthServerMutationPrecondition,
    request::WorthServerCompatibilityMutationRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerCompatibilityMutation {
    operation_request: crate::WorthServerOperationRequest,
    plan_proof: crate::WorthServerOperationPlanProof,
    mutation_request: WorthServerCompatibilityMutationRequest,
    precondition: WorthServerMutationPrecondition,
    mutation_result: WorthServerCompatibilityMutationResult,
    envelope: WorthServerCompatibilityMutationEnvelope,
    canonical_digest: String,
}

impl WorthServerCompatibilityMutation {
    pub(crate) fn new(
        operation_request: crate::WorthServerOperationRequest,
        plan_proof: crate::WorthServerOperationPlanProof,
        mutation_request: WorthServerCompatibilityMutationRequest,
        precondition: WorthServerMutationPrecondition,
        mutation_result: WorthServerCompatibilityMutationResult,
        envelope: WorthServerCompatibilityMutationEnvelope,
    ) -> Self {
        let canonical_digest = format!(
            "worth-server-compat-mutation-v1|operation_request:{}|request:{}|precondition:{}|result:{}|inspection:{}|envelope:{}",
            operation_request.canonical_digest(),
            mutation_request.canonical_digest(),
            precondition.canonical_digest(),
            mutation_result.result_digest(),
            mutation_result.inspection_digest(),
            envelope.canonical_digest(),
        );
        Self {
            operation_request,
            plan_proof,
            mutation_request,
            precondition,
            mutation_result,
            envelope,
            canonical_digest,
        }
    }

    pub fn operation_request(&self) -> &crate::WorthServerOperationRequest {
        &self.operation_request
    }

    pub fn plan_proof(&self) -> &crate::WorthServerOperationPlanProof {
        &self.plan_proof
    }

    pub fn mutation_request(&self) -> &WorthServerCompatibilityMutationRequest {
        &self.mutation_request
    }

    pub fn precondition(&self) -> &WorthServerMutationPrecondition {
        &self.precondition
    }

    pub fn mutation_result(&self) -> &WorthServerCompatibilityMutationResult {
        &self.mutation_result
    }

    pub fn envelope(&self) -> &WorthServerCompatibilityMutationEnvelope {
        &self.envelope
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerCompatibilityMutationResult {
    Single {
        receipt: Box<WorthQueryWriteReceipt>,
        inspection: Box<WorthQueryWriteReceiptInspection>,
    },
    Batch {
        receipt: Box<WorthQueryBatchWriteReceipt>,
        inspection: Box<WorthQueryBatchWriteReceiptInspection>,
    },
}

impl WorthServerCompatibilityMutationResult {
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

    pub fn result_digest(&self) -> &str {
        match self {
            Self::Single { receipt, .. } => receipt
                .commit_evidence_identity()
                .terminal_projection_for_reporting(),
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
