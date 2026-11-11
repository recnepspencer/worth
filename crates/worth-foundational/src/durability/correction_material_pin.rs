//! Stable identity and handoff meaning of one retained correction-material use.
use super::{
    identity_seal, DurableCompositePublicationAttemptIdentity, DurableIdentity, HomeIdentity,
};
use std::num::NonZeroU64;

/// Journal DATA; constructing it never claims a live pin or authenticates a Home.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct CorrectionMaterialPinIdentity {
    home: HomeIdentity,
    acquire_position: NonZeroU64,
}
impl CorrectionMaterialPinIdentity {
    pub const fn from_recorded(home: HomeIdentity, acquire_position: NonZeroU64) -> Self {
        Self {
            home,
            acquire_position,
        }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn acquire_position(self) -> NonZeroU64 {
        self.acquire_position
    }
}
impl identity_seal::Sealed for CorrectionMaterialPinIdentity {}
impl DurableIdentity for CorrectionMaterialPinIdentity {}

/// The same mandatory use is handed from a disclosed command to its actual open.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CorrectionMaterialPinOwner {
    PreparedCommand {
        key: [u8; 32],
    },
    Publication {
        original: DurableCompositePublicationAttemptIdentity,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectionMaterialPinTerminal {
    Released,
    OwnerAbandoned,
    Transferred(CorrectionMaterialPinOwner),
}
impl worth_proof::TerminalState for CorrectionMaterialPinTerminal {
    fn label(&self) -> &'static str {
        match self {
            Self::Released => "released",
            Self::OwnerAbandoned => "owner-abandoned",
            Self::Transferred(_) => "transferred",
        }
    }
}
