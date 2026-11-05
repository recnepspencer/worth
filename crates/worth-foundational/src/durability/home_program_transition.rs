//! Recorded Home migration identities. None grants live migration authority.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;

/// The original intent's position in this Home's `home.transitions` journal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HomeProgramTransitionIdentity {
    home: HomeIdentity,
    intent: NonZeroU64,
}
impl HomeProgramTransitionIdentity {
    pub const fn from_recorded(home: HomeIdentity, intent: NonZeroU64) -> Self {
        Self { home, intent }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn intent_position(self) -> NonZeroU64 {
        self.intent
    }
}
impl identity_seal::Sealed for HomeProgramTransitionIdentity {}
impl DurableIdentity for HomeProgramTransitionIdentity {}

/// A zero-based member of the complete plan bound by the original intent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HomeProgramTransitionStepAssociation {
    transition: HomeProgramTransitionIdentity,
    step: u32,
}
impl HomeProgramTransitionStepAssociation {
    pub const fn from_recorded(transition: HomeProgramTransitionIdentity, step: u32) -> Self {
        Self { transition, step }
    }
    pub const fn transition(self) -> HomeProgramTransitionIdentity {
        self.transition
    }
    pub const fn step(self) -> u32 {
        self.step
    }
}
impl identity_seal::Sealed for HomeProgramTransitionStepAssociation {}
impl DurableIdentity for HomeProgramTransitionStepAssociation {}

/// Canonical commitment to every step, its order and its dispositions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionPlanDigest([u8; 32]);
impl HomeProgramTransitionPlanDigest {
    pub const fn from_recorded(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
impl identity_seal::Sealed for HomeProgramTransitionPlanDigest {}
impl DurableIdentity for HomeProgramTransitionPlanDigest {}

/// Canonical commitment to one complete authored Native transaction body.
/// Recorded bytes are data, never permission to execute that body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionBodyDigest([u8; 32]);
impl HomeProgramTransitionBodyDigest {
    pub const fn from_recorded(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
impl identity_seal::Sealed for HomeProgramTransitionBodyDigest {}
impl DurableIdentity for HomeProgramTransitionBodyDigest {}
