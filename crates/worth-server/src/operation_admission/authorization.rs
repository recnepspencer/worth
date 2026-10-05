use crate::WorthServerAdmission;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationAuthorizationProof {
    admission: WorthServerAdmission,
    operation_reference_digest: String,
    footprint_digest: String,
    authorization_lane: String,
    canonical_digest: String,
}

impl WorthServerOperationAuthorizationProof {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        self.admission
            .owned_allocation_capacity_bytes()
            .saturating_add(string(&self.operation_reference_digest))
            .saturating_add(string(&self.footprint_digest))
            .saturating_add(string(&self.authorization_lane))
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(
        admission: WorthServerAdmission,
        operation_reference_digest: impl Into<String>,
        footprint_digest: impl Into<String>,
        authorization_lane: impl Into<String>,
    ) -> Self {
        let operation_reference_digest = operation_reference_digest.into();
        let footprint_digest = footprint_digest.into();
        let authorization_lane = authorization_lane.into();
        let canonical_digest = format!(
            "worth-server-operation-authorization-proof-v5|operation_reference={operation_reference_digest}|footprint={footprint_digest}|lane={authorization_lane}",
        );
        Self {
            admission,
            operation_reference_digest,
            footprint_digest,
            authorization_lane,
            canonical_digest,
        }
    }

    pub fn admission(&self) -> &WorthServerAdmission {
        &self.admission
    }

    pub fn operation_reference_digest(&self) -> &str {
        &self.operation_reference_digest
    }

    pub fn authorization_lane(&self) -> &str {
        &self.authorization_lane
    }

    pub fn footprint_digest(&self) -> &str {
        &self.footprint_digest
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
