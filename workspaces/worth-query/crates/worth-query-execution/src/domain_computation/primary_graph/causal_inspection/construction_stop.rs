use super::{
    CausalEvidenceReferenceIndexError, CausalInspectionMaterializationError,
    CausalInspectionRequestError, CausalObservationAnchorError,
};
use crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial;

/// Construction preserves a digest refusal even when it happens while building
/// another diagnostic failure. No partially constructed identity is published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalInspectionConstructionStop {
    Digest(WorthQueryEvidenceIdentityDerivationDenial),
    Bridge(worth_runtime_bridge::facade::BridgeCausalEnvelopeConstructionStop),
    Anchor(CausalObservationAnchorError),
    Request(CausalInspectionRequestError),
    ReferenceIndex(CausalEvidenceReferenceIndexError),
    Materialization(CausalInspectionMaterializationError),
}

impl From<WorthQueryEvidenceIdentityDerivationDenial> for CausalInspectionConstructionStop {
    fn from(cause: WorthQueryEvidenceIdentityDerivationDenial) -> Self {
        Self::Digest(cause)
    }
}
impl From<CausalObservationAnchorError> for CausalInspectionConstructionStop {
    fn from(cause: CausalObservationAnchorError) -> Self {
        Self::Anchor(cause)
    }
}
impl From<CausalInspectionRequestError> for CausalInspectionConstructionStop {
    fn from(cause: CausalInspectionRequestError) -> Self {
        Self::Request(cause)
    }
}
impl From<CausalEvidenceReferenceIndexError> for CausalInspectionConstructionStop {
    fn from(cause: CausalEvidenceReferenceIndexError) -> Self {
        Self::ReferenceIndex(cause)
    }
}
impl From<CausalInspectionMaterializationError> for CausalInspectionConstructionStop {
    fn from(cause: CausalInspectionMaterializationError) -> Self {
        Self::Materialization(cause)
    }
}
