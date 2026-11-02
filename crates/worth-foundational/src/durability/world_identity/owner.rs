use super::super::{identity_seal, DurableIdentity, HomeIdentity};
/// The stable World owner slot recorded inside a Home, not a process ordinal.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct WorldOwnerSlotIdentity(u64);
impl WorldOwnerSlotIdentity {
    /// Descriptive recorded data; this grants no attachment or issuer authority.
    pub const fn from_recorded(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl identity_seal::Sealed for WorldOwnerSlotIdentity {}
impl DurableIdentity for WorldOwnerSlotIdentity {}
/// World identity whose scope remains the same across process reopen.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct DurableWorldOwnerIdentity {
    home: HomeIdentity,
    slot: WorldOwnerSlotIdentity,
}
impl DurableWorldOwnerIdentity {
    /// Reading data cannot claim the Home's exclusive World attachment.
    pub const fn from_recorded(home: HomeIdentity, slot: WorldOwnerSlotIdentity) -> Self {
        Self { home, slot }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn slot(self) -> WorldOwnerSlotIdentity {
        self.slot
    }
}
impl identity_seal::Sealed for DurableWorldOwnerIdentity {}
impl DurableIdentity for DurableWorldOwnerIdentity {}
