use super::{BridgeCausalDigestRefusal, BridgeCausalEnvelopeDenial};

/// A failed envelope construction retains either the original domain denial
/// or a refused digest, including refusal while constructing denial evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeCausalEnvelopeConstructionStop {
    Digest(BridgeCausalDigestRefusal),
    Denied(BridgeCausalEnvelopeDenial),
}
impl From<BridgeCausalDigestRefusal> for BridgeCausalEnvelopeConstructionStop {
    fn from(cause: BridgeCausalDigestRefusal) -> Self {
        Self::Digest(cause)
    }
}
impl From<BridgeCausalEnvelopeDenial> for BridgeCausalEnvelopeConstructionStop {
    fn from(cause: BridgeCausalEnvelopeDenial) -> Self {
        Self::Denied(cause)
    }
}
