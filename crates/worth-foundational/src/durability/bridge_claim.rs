//! A Bridge claim originates in one Signal-authored definition transition.
use super::{identity_seal, DurableIdentity, SignalDefinitionTransitionIdentity};

/// Stable source reference, not a live Bridge or Native admission.
/// The transition's owning issuer resumes from all recorded Signal bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BridgeSemanticClaimReference {
    definition: SignalDefinitionTransitionIdentity,
    ordinal: u32,
}
impl BridgeSemanticClaimReference {
    pub const fn from_recorded(
        definition: SignalDefinitionTransitionIdentity,
        ordinal: u32,
    ) -> Self {
        Self {
            definition,
            ordinal,
        }
    }
    pub const fn definition(self) -> SignalDefinitionTransitionIdentity {
        self.definition
    }
    pub const fn ordinal(self) -> u32 {
        self.ordinal
    }
}
impl identity_seal::Sealed for BridgeSemanticClaimReference {}
impl DurableIdentity for BridgeSemanticClaimReference {}
