use crate::WorthServerOperationRequest;

use super::{
    WorthServerOperationAuthorityFootprint, WorthServerOperationAuthorityMetadata,
    WorthServerOperationAuthorizationProof, WorthServerOperationFootprintReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationAdmissionPosture {
    operation_request: WorthServerOperationRequest,
    authority_metadata: WorthServerOperationAuthorityMetadata,
    authority_footprint: WorthServerOperationAuthorityFootprint,
    footprint_receipt: WorthServerOperationFootprintReceipt,
    authorization_proof: WorthServerOperationAuthorizationProof,
    canonical_digest: String,
}

impl WorthServerOperationAdmissionPosture {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        self.operation_request
            .owned_allocation_capacity_bytes()
            .saturating_add(self.authority_metadata.owned_allocation_capacity_bytes())
            .saturating_add(self.authority_footprint.owned_allocation_capacity_bytes())
            .saturating_add(self.footprint_receipt.owned_allocation_capacity_bytes())
            .saturating_add(self.authorization_proof.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(
        operation_request: WorthServerOperationRequest,
        authority_metadata: WorthServerOperationAuthorityMetadata,
        authority_footprint: WorthServerOperationAuthorityFootprint,
        footprint_receipt: WorthServerOperationFootprintReceipt,
        authorization_proof: WorthServerOperationAuthorizationProof,
    ) -> Self {
        let canonical_digest = format!(
            "worth-server-operation-admission-posture-v4|metadata={}|footprint={}|receipt={}|authorization={}",
            authority_metadata.canonical_digest(),
            authority_footprint.canonical_digest(),
            footprint_receipt.canonical_digest(),
            authorization_proof.canonical_digest(),
        );
        Self {
            operation_request,
            authority_metadata,
            authority_footprint,
            footprint_receipt,
            authorization_proof,
            canonical_digest,
        }
    }

    pub fn operation_request(&self) -> &WorthServerOperationRequest {
        &self.operation_request
    }

    pub fn authority_metadata(&self) -> &WorthServerOperationAuthorityMetadata {
        &self.authority_metadata
    }

    pub fn authority_footprint(&self) -> &WorthServerOperationAuthorityFootprint {
        &self.authority_footprint
    }

    pub fn footprint_receipt(&self) -> &WorthServerOperationFootprintReceipt {
        &self.footprint_receipt
    }

    pub fn authorization_proof(&self) -> &WorthServerOperationAuthorizationProof {
        &self.authorization_proof
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
